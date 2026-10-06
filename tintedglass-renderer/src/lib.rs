mod error;
mod viewport;

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

pub use error::{Error, Result};

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use tintedglass_model::{BlockId, Litematic};

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

#[derive(Component)]
pub struct Block(pub BlockId);

fn spawn_schematic(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut orbit: Single<&mut OrbitCamera>,
    schematic: Res<Schematic>,
) {
    let schematic = &schematic.0;

    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let materials: HashMap<_, _> = schematic
        .palettes()
        .map(|(block_id, palette)| {
            let mut hasher = DefaultHasher::new();
            palette.name.hash(&mut hasher);
            let hue = (hasher.finish() % 360) as f32;
            let material = StandardMaterial {
                base_color: Color::hsl(hue, 0.6, 0.5),
                ..default()
            };
            (block_id, materials.add(material))
        })
        .collect();

    for (pos, block_id) in schematic
        .blocks_dedup()
        .filter(|(_, block_id)| block_id.id() != 0)
    {
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(materials.get(&block_id).unwrap().clone()),
            Transform::from_translation(pos.as_vec3() + 0.5),
        ));
    }

    let (min_corner, max_corner) = (schematic.min_corner(), schematic.max_corner());
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

fn create_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Y; 4])
    .with_inserted_indices(Indices::U32(vec![0, 1, 3, 3, 1, 2]))
}
