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
    commands.spawn((InfiniteGrid, InfiniteGridSettings::default()));
}
