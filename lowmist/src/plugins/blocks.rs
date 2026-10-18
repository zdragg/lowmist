mod mesher;
pub mod resource_pack;
pub mod schematic;

use std::sync::Arc;

use bevy::{platform::collections::HashMap, prelude::*};
use flume::{Receiver, Sender};
use lowmist_schem::Litematic;

use crate::plugins::blocks::{
    resource_pack::InnerMinecraftAssetPlugin,
    schematic::{InnerSchematicPlugin, LoadedSchematic, Schematic},
};

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InnerSchematicPlugin)
            .add_plugins(InnerMinecraftAssetPlugin)
            .add_message::<NewBlocksLoadingMessage>()
            .add_systems(Startup, spawn_threads)
            .add_systems(
                Update,
                (
                    spawn_schematic_root_on_new_loaded_schematic,
                    spawn_schematic_children,
                ),
            );
    }
}

#[derive(Resource)]
struct ChunkMeshChannel {
    job_tx: Sender<(Arc<Litematic>, IVec3)>,
    result_rx: Receiver<(IVec3, Mesh)>,
}

const PERCENT_OF_TOTAL_THREADS: f32 = 0.25;
const MIN_THREADS: usize = 1;
const MAX_THREADS: usize = 4;

const CHANNEL_BOUND: usize = 64;

/// Spawns background meshing threads that take jobs and return meshes
fn spawn_threads(mut commands: Commands) {
    #[cfg(not(target_family = "wasm"))]
    use std::thread;
    #[cfg(target_family = "wasm")]
    use wasm_thread as thread;

    // Unbounded because main thread cannot wait for workers to receive
    let (job_tx, job_rx) = flume::unbounded::<(Arc<Litematic>, IVec3)>();
    // Bounded to decrease the amount of meshes waiting to be loaded in memory.
    let (result_tx, result_rx) = flume::bounded::<(IVec3, Mesh)>(CHANNEL_BOUND);

    let total_threads = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let desired = (total_threads as f32 * PERCENT_OF_TOTAL_THREADS).round() as usize;
    let thread_count = desired.clamp(MIN_THREADS, MAX_THREADS);

    for _ in 0..thread_count {
        let job_rx = job_rx.clone();
        let result_tx = result_tx.clone();
        thread::spawn(move || {
            // Thread needs to wait for the next task, so use blocking iter()
            for (schem, chunk_pos) in job_rx.iter() {
                let mesh = mesher::build_chunk_mesh(&schem, chunk_pos);
                if let Some(mesh) = mesh {
                    let _ = result_tx.send((chunk_pos, mesh));
                }
            }
        });
    }

    commands.insert_resource(ChunkMeshChannel { job_tx, result_rx });
}

// Do not make this component public.
// The sole purpose for this to hold the handle is for a way to check
// whether the currently spawned schematic entity and the schematic stored
// within LoadedSchematic above is the same schematic or not.
//
// If you want to get access to the schematic, do it through the LoadedSchematic Resource.
#[derive(Component)]
struct SchematicRoot(Handle<Schematic>);

#[derive(Component)]
struct StandardMaterialHandle(Handle<StandardMaterial>);

/// A replacement for Children that allows O(1) chunk_pos -> Entity
#[derive(Component)]
struct ChunkEntities(HashMap<IVec3, Entity>);
impl ChunkEntities {
    fn new() -> Self {
        Self(HashMap::new())
    }
}

/// A message that indicates a successful schematic spawn and includes the schemaitc bounds.
///
/// Could be used to help Cameras initialize itself with a reasonable Transform.
#[derive(Message)]
pub struct NewBlocksLoadingMessage {
    /// The schematic's corner with the smallest number on each axis
    pub min_corner: IVec3,
    /// The schematic's corner with the largest number on each axis
    pub max_corner: IVec3,
}

fn spawn_schematic_root_on_new_loaded_schematic(
    mut commands: Commands,

    mut materials: ResMut<Assets<StandardMaterial>>,

    // Schematic
    rendered_schem: Option<Single<(Entity, &SchematicRoot)>>,
    loaded_assets: Res<Assets<Schematic>>,
    loaded_handle_resource: Res<LoadedSchematic>,

    channel: Res<ChunkMeshChannel>,

    mut message_writer: MessageWriter<NewBlocksLoadingMessage>,
) {
    // A "Loaded" schematic is one currently loaded within the asset server.
    // A "Rendered" schematic is the one being rendered in the world.

    let loaded_handle = &loaded_handle_resource.0;
    let Some(loaded_asset) = loaded_assets.get(loaded_handle) else {
        return; // No Loaded schematic. Nothing to do.
    };

    if let Some(rendered_schem) = rendered_schem {
        // A schematic is being Rendered.
        let (rendered_entity, rendered_root) = *rendered_schem;
        let rendered_schem_handle = &rendered_root.0; // Obtain the asset handle for the Rendered schematic.

        if rendered_schem_handle == loaded_handle {
            return; // Rendered == Loaded. Nothing to do.
        }

        // Otherwise, a different schematic is Loaded. Despawn the Rendered schematic entity.
        commands.entity(rendered_entity).despawn();
    }

    let schematic = loaded_asset.inner();

    for chunk_pos in schematic.chunk_positions() {
        let _ = channel.job_tx.send((schematic.clone(), chunk_pos));
    }

    // Broadcast schematic bounds when the schematic STARTS loading
    let (min_corner, max_corner) = schematic.bounds();
    message_writer.write(NewBlocksLoadingMessage {
        min_corner,
        max_corner,
    });

    commands.spawn((
        SchematicRoot(loaded_handle_resource.0.clone()), // Schematic asset handle stored here
        ChunkEntities::new(),                            // The rendered chunk entities
        StandardMaterialHandle(materials.add(StandardMaterial::default())), // The shared standard material
        Transform::IDENTITY,
        Visibility::default(),
    ));
}

const MESHES_UPLOADED_PER_FRAME: usize = 16;

fn spawn_schematic_children(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,

    channels: Res<ChunkMeshChannel>,
    schem_parent: Single<
        (Entity, &StandardMaterialHandle, Mut<ChunkEntities>),
        With<SchematicRoot>,
    >,
) {
    let (root, material, mut entities) = schem_parent.into_inner();

    for (chunk_pos, mesh) in channels
        .result_rx
        .try_iter()
        .take(MESHES_UPLOADED_PER_FRAME)
    {
        match entities.0.get(&chunk_pos) {
            // Spawn new entity and insert
            None => {
                let entity = commands
                    .spawn((
                        ChildOf(root),
                        Mesh3d(meshes.add(mesh)),
                        MeshMaterial3d(material.0.clone()),
                        Transform::from_translation((chunk_pos * 16).as_vec3()),
                    ))
                    .id();
                entities.0.insert(chunk_pos, entity);
            }
            // Replace only the Mesh3d component on the entity
            Some(entity) => {
                let mesh_handle = meshes.add(mesh);
                commands.entity(*entity).insert_if_neq(Mesh3d(mesh_handle));
            }
        }
    }
}
