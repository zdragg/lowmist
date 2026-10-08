use bevy::{
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    prelude::*,
};

pub struct ChunkGridPlugin;

impl Plugin for ChunkGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, spawn_grid)
            .add_systems(Update, shift_grid_to_prevent_z_fighting);
    }
}

fn spawn_grid(mut commands: Commands) {
    // Chunk grid
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            scale: 1.0 / 16.0,
            minor_line_color: Color::srgb(0.25, 0.25, 0.25),
            major_line_color: Color::srgb(0.25, 0.25, 0.25),
            fadeout_distance: 1000.0,
            ..default()
        },
    ));

    // Block grid
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            scale: 1.0,
            minor_line_color: Color::srgb(0.2, 0.2, 0.2),
            major_line_color: Color::srgb(0.2, 0.2, 0.2),
            fadeout_distance: 1000.0,
            ..default()
        },
    ));
}

fn shift_grid_to_prevent_z_fighting(
    camera: Option<Single<&Transform, With<Camera3d>>>,
    mut grid_transforms: Query<&mut Transform, (With<InfiniteGrid>, Without<Camera3d>)>,
) {
    let Some(camera) = camera else {
        return;
    };

    let y = if camera.translation.y.is_sign_positive() {
        0.001
    } else {
        -0.001
    };

    for mut transform in &mut grid_transforms {
        transform.translation.y = y;
    }
}
