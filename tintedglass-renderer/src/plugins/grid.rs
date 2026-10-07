use bevy::{
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    prelude::*,
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, spawn_grid);
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
