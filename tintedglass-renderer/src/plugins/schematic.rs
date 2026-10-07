mod mesher;

use bevy::prelude::*;
use tintedglass_model::Litematic;

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SchematicSpawned>().add_systems(
            Update,
            spawn_schematic.run_if(resource_exists_and_changed::<Schematic>),
        );
    }
}

#[derive(Resource)]
pub struct Schematic(pub Litematic);

/// A message that indicates a successful schematic spawn and includes the schemaitc bounds.
///
/// Use this to initialize the camera with reasonable numbers.
#[derive(Message)]
pub struct SchematicSpawned {
    /// The schematic's corner with the smallest number on each axis
    pub min_corner: IVec3,
    /// The schematic's corner with the largest number on each axis
    pub max_corner: IVec3,
}

#[derive(Component)]
struct SchematicRoot;

fn spawn_schematic(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut message_writer: MessageWriter<SchematicSpawned>,
    schematic: Res<Schematic>,
) {
    let schematic = &schematic.0;
    let standard_material = materials.add(StandardMaterial { ..default() });

    commands
        .spawn((SchematicRoot, Transform::IDENTITY, Visibility::default()))
        .with_children(|parent| {
            for (chunk_pos, mesh) in mesher::build_chunked_meshes(schematic) {
                parent.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(standard_material.clone()),
                    Transform::from_translation((chunk_pos * 16).as_vec3()),
                ));
            }
        });

    let (min_corner, max_corner) = schematic.bounds();

    // Broadcast schematic spawn along with schematic bounds
    message_writer.write(SchematicSpawned {
        min_corner,
        max_corner,
    });
}
