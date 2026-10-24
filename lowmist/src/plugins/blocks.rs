mod mesher;
pub mod resource_pack;
pub mod schematic;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering::Relaxed},
};

use bevy::prelude::*;
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
            .add_systems(Startup, spawn_root_entity)
            .add_systems(
                Update,
                (load_new_schematic, send_mesh_jobs, receive_mesh_results),
            );
    }
}

#[derive(Resource)]
struct MeshJobManager {
    generation: Arc<AtomicUsize>,
    chunk_positions: Arc<ChunkPositions>,
    current_schematic: Arc<Litematic>,
    in_flight: Arc<AtomicUsize>,
    result_tx: Sender<(usize, IVec3, Mesh)>,
    result_rx: Receiver<(usize, IVec3, Mesh)>,
}

impl MeshJobManager {
    fn new(schematic: Arc<Litematic>) -> Self {
        let (result_tx, result_rx) = flume::unbounded::<(usize, IVec3, Mesh)>();
        Self {
            generation: Arc::new(AtomicUsize::new(0)),
            chunk_positions: Arc::new(ChunkPositions::new(schematic.chunk_positions())),
            current_schematic: schematic,
            in_flight: Arc::new(AtomicUsize::new(0)),
            result_tx,
            result_rx,
        }
    }

    /// Returns false if new_schematic is same as current schematic.
    /// Otherwise, returns true.
    fn try_swap_schematic(&mut self, new_schematic: Arc<Litematic>) -> bool {
        if Arc::ptr_eq(&self.current_schematic, &new_schematic) {
            return false; // Already equal
        }
        self.generation.fetch_add(1, Relaxed);
        self.chunk_positions = Arc::new(ChunkPositions::new(new_schematic.chunk_positions()));
        self.current_schematic = new_schematic;
        true
    }
}

struct ChunkPositions {
    chunk_position_cursor: AtomicUsize,
    chunk_positions: Box<[IVec3]>,
}

impl ChunkPositions {
    fn new(chunk_positions: impl IntoIterator<Item = IVec3>) -> Self {
        Self {
            chunk_position_cursor: AtomicUsize::new(0),
            chunk_positions: chunk_positions.into_iter().collect(),
        }
    }
    fn claim_next(&self) -> Option<IVec3> {
        let index = self.chunk_position_cursor.fetch_add(1, Relaxed);
        self.chunk_positions.get(index).copied()
    }
    fn done(&self) -> bool {
        self.chunk_position_cursor.load(Relaxed) >= self.chunk_positions.len()
    }
}

#[derive(Component)]
struct SchematicRoot;

#[derive(Component)]
struct StandardMaterialHandle(Handle<StandardMaterial>);

fn spawn_root_entity(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        SchematicRoot,
        StandardMaterialHandle(materials.add(StandardMaterial::default())), // The shared standard material
        Transform::IDENTITY,
        Visibility::default(),
    ));
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

fn load_new_schematic(
    mut commands: Commands,
    root_entity: Single<Entity, With<SchematicRoot>>,
    mesh_manager: Option<ResMut<MeshJobManager>>,
    schem_server: Res<Assets<Schematic>>,
    loaded_schem_handle: Res<LoadedSchematic>,
    mut new_blocks_message_writer: MessageWriter<NewBlocksLoadingMessage>,
) {
    let Some(new_schematic) = schem_server.get(&loaded_schem_handle.0) else {
        return; // No Loaded schematic. Nothing to do.
    };
    let new_schematic = new_schematic.get();

    let (min_corner, max_corner) = new_schematic.bounds();
    let new_blocks_loading_message = NewBlocksLoadingMessage {
        min_corner,
        max_corner,
    };

    match mesh_manager {
        Some(mut manager) => {
            if manager.try_swap_schematic(new_schematic) {
                commands.entity(root_entity.entity()).despawn_children();
                new_blocks_message_writer.write(new_blocks_loading_message);
            }
        }
        None => {
            commands.insert_resource(MeshJobManager::new(new_schematic));
            new_blocks_message_writer.write(new_blocks_loading_message);
        }
    };
}

const MESHES_CAP: usize = 64;
fn send_mesh_jobs(mesh_manager: If<ResMut<MeshJobManager>>) {
    while mesh_manager.in_flight.load(Relaxed) <= MESHES_CAP {
        if mesh_manager.chunk_positions.done() {
            return;
        }

        let schematic = mesh_manager.current_schematic.clone();
        let in_flight = mesh_manager.in_flight.clone();
        let result_tx = mesh_manager.result_tx.clone();
        let chunk_positions = mesh_manager.chunk_positions.clone();

        let schematic_gen = mesh_manager.generation.load(Relaxed);
        let current_gen = mesh_manager.generation.clone();

        in_flight.fetch_add(1, Relaxed);
        rayon::spawn(move || {
            while let Some(chunk_pos) = chunk_positions.claim_next() {
                if current_gen.load(Relaxed) != schematic_gen {
                    // A new schematic has spawned, do nothing
                    in_flight.fetch_sub(1, Relaxed);
                    return;
                }

                if let Some(mesh) = mesher::build_chunk_mesh(&schematic, chunk_pos) {
                    let _ = result_tx.send((schematic_gen, chunk_pos, mesh));
                    return;
                }
            }

            // No more chunk positions to claim
            in_flight.fetch_sub(1, Relaxed);
        });
    }
}

fn receive_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_manager: If<Res<MeshJobManager>>,
    schematic_root: Single<(Entity, &StandardMaterialHandle), With<SchematicRoot>>,
) {
    let (root_entity, shared_material) = schematic_root.into_inner();

    for (generation, chunk_pos, mesh) in mesh_manager.result_rx.drain() {
        mesh_manager.in_flight.fetch_sub(1, Relaxed);
        if mesh_manager.generation.load(Relaxed) != generation {
            continue; // new schematic loaded during meshing
        }
        commands.spawn((
            ChildOf(root_entity),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(shared_material.0.clone()),
            Transform::from_translation((chunk_pos * 16).as_vec3()),
        ));
    }
}
