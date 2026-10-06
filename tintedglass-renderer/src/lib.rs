mod error;
mod mesher;
mod viewport;

pub use error::{Error, Result};

use bevy::prelude::*;
use tintedglass_model::Litematic;

use crate::viewport::{
    camera::{MinecraftCamera, MinecraftCameraPlugin},
    grid::GridPlugin,
};

pub struct TintedGlassPlugin;

impl Plugin for TintedGlassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MinecraftCameraPlugin)
            .add_plugins(GridPlugin)
            .add_systems(
                Update,
                spawn_schematic.run_if(resource_exists_and_changed::<Schematic>),
            );
    }
}

#[derive(Resource)]
pub struct Schematic(pub Litematic);

fn spawn_schematic(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera: Single<(&mut Transform, &mut MinecraftCamera)>,
    schematic: Res<Schematic>,
) {
    let schematic = &schematic.0;

    let mesh = mesher::build_mesh(schematic);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::IDENTITY,
    ));

    let (transform, camera) = &mut *camera;

    let (min_corner, max_corner) = schematic.bounds();
    let center_pos = (min_corner + max_corner + 1).as_vec3() / 2.0;
    // The radius of the smallest sphere that encompasses the entire schematic
    let radius = (max_corner - min_corner + 1).as_vec3().length() / 2.0;
    // Half of the camera's field of view (default camera FOV is PI/4)
    let half_fov = std::f32::consts::FRAC_PI_8;
    // Minimum distance required to fit the entire schematic/sphere
    let minimum_distance = radius / half_fov.sin();

    // Move toward this direction (upwards and between +x and +z)
    // for minimum_distance,
    // and look back toward center_pos
    let direction = Vec3::new(1.0, 0.6, 1.0).normalize();
    **transform = Transform::from_translation(center_pos + direction * minimum_distance)
        .looking_at(center_pos, Vec3::Y);
    // And set camera to the same rotation
    let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
    camera.set_rotation(yaw, pitch);
}
