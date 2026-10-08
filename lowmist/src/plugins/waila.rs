use bevy::prelude::*;
use lowmist_schematic::BlockStatePaletteEntry;
use voxel_traversal::VoxelRaycast;

use crate::plugins::{
    camera::MinecraftCamera,
    schematic::{LoadedSchematic, SchematicAsset, SchematicSpawned},
};

pub struct WailaPlugin;

impl Plugin for WailaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PointedBlock>()
            .add_systems(Startup, (spawn_waila, spawn_crosshair))
            .add_systems(
                Update,
                (
                    find_pointed_block,
                    update_waila_text,
                    update_crosshair,
                    draw_block_outline,
                ),
            );
    }
}

const REACH: f32 = 1000.;

#[derive(Default, PartialEq, Resource)]
struct PointedBlock(Option<(IVec3, BlockStatePaletteEntry)>);

/// Find the currently pointed block and store it into PointedBlock.
/// Store None into the Resource if currently looking at no blocks within REACH.
fn find_pointed_block(
    schem_handle: Res<LoadedSchematic>,
    schem_asset_storage: Res<Assets<SchematicAsset>>,
    camera: Option<Single<Ref<Transform>, With<MinecraftCamera>>>,
    mut events: MessageReader<SchematicSpawned>,

    mut pointed_block: ResMut<PointedBlock>,
) {
    let Some(camera) = camera else {
        return; // A camera has to exist
    };

    let Some(schem_asset) = schem_asset_storage.get(&schem_handle.0) else {
        return; // A schematic must be Loaded
    };

    let new_schem_spawned = events.read().last().is_some(); // Drain the reader queue
    if !camera.is_changed() && !new_schem_spawned {
        return;
    }

    let schem = schem_asset.inner();

    let start_pos = camera.translation;
    let direction = camera.forward();
    let end_pos = start_pos + direction * REACH;

    // Find the first non air block on the ray from start_pos to end_pos
    let block = VoxelRaycast::new(start_pos, end_pos).find_map(|((x, y, z), _normals)| {
        let pos = IVec3::from_array([x, y, z]);
        let block_id = schem.block_at(pos)?;
        (!block_id.is_air()).then_some((pos, schem.palette_entry(block_id).clone()))
    });

    pointed_block.set_if_neq(PointedBlock(block));
}

#[derive(Component)]
struct WailaText;

fn spawn_waila(mut commands: Commands) {
    commands.spawn((
        WailaText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn update_waila_text(
    pointed_block: Res<PointedBlock>,
    mut text: Single<Mut<Text>, With<WailaText>>,
) {
    if pointed_block.is_changed() {
        match &pointed_block.0 {
            None => text.0.clear(),
            Some((_pos, block)) => text.0.clone_from(&block.name),
        }
    }
}

fn draw_block_outline(pointed_block: Res<PointedBlock>, mut gizmos: Gizmos) {
    if let Some((pos, _block)) = &pointed_block.0 {
        gizmos.cube(
            Transform::from_translation(pos.as_vec3() + 0.5),
            Color::srgba(0.0, 0.0, 0.0, 1.0),
        );
    }
}

#[derive(Component)]
struct Crosshair;

fn spawn_crosshair(mut commands: Commands) {
    commands
        .spawn((Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_child((Crosshair, Text::new("+")));
}

fn update_crosshair(
    pointed_block: Res<PointedBlock>,
    mut visibility: Single<Mut<Visibility>, With<Crosshair>>,
) {
    if pointed_block.is_changed() {
        let new_visibility = match pointed_block.0 {
            Some(_) => Visibility::Visible,
            None => Visibility::Hidden,
        };
        visibility.set_if_neq(new_visibility);
    }
}
