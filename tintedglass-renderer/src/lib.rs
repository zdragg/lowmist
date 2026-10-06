mod error;
mod mesher;
mod viewport;

pub use error::{Error, Result};

use bevy::prelude::*;
use tintedglass_model::Litematic;

use crate::viewport::{
    camera::{OrbitCamera, OrbitCameraPlugin},
    grid::GridPlugin,
};

pub struct TintedGlassPlugin;

impl Plugin for TintedGlassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OrbitCameraPlugin)
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
    mut orbit: Single<&mut OrbitCamera>,
    schematic: Res<Schematic>,
) {
    let schematic = &schematic.0;

    let mesh = mesher::build_mesh(schematic);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::IDENTITY,
    ));

    let (min_corner, max_corner) = schematic.bounds();
    let center_pos = (min_corner + max_corner + 1).as_vec3() / 2.0;
    orbit.focus = center_pos;
    // The radius of the smallest sphere that encompasses the entire schematic
    let radius = (max_corner - min_corner + 1).as_vec3().length() / 2.0;
    // Half of the camera's field of view (default camera FOV is PI/4)
    let half_fov = std::f32::consts::FRAC_PI_8;
    // Minimum distance required to fit the entire schematic/sphere
    let minimum_distance = radius / half_fov.sin();
    orbit.distance = minimum_distance;
}
