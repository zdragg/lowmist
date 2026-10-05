mod error;
mod viewport;

pub use error::{Error, Result};

use bevy::prelude::*;
use tintedglass_model::{BlockId, Litematic};

use crate::viewport::{camera::OrbitCameraPlugin, grid::GridPlugin};

pub struct TintedGlassPlugin;

impl Plugin for TintedGlassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OrbitCameraPlugin)
            .add_plugins(GridPlugin)
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                spawn_schematic.run_if(resource_exists_and_changed::<Schematic>),
            );
    }
}

#[derive(Resource)]
pub struct Schematic(pub Litematic);

#[derive(Component)]
pub struct Block(pub BlockId);

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.0,
            ..default()
        },
        Transform::IDENTITY.looking_to(Vec3::new(-1.0, -2.0, -1.0).normalize(), Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.7),
            perceptual_roughness: 0.85,
            ..default()
        })),
        Transform::from_xyz(0.5, 0.5, 0.5),
    ));
}

fn spawn_schematic(mut commands: Commands, schematic: Res<Schematic>) {
    // TODO: despawn the previous schematic's blocks
    // TODO: one material per BlockId, from schematic.0.palettes()
    // TODO: spawn a cube per non-air block from schematic.0.blocks_dedup()
    // TODO: point the camera at the schematic's bounds
}
