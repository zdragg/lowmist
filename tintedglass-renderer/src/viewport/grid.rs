use bevy::{
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    prelude::*,
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, grid.spawn());
    }
}

fn grid() -> impl Scene {
    bsn! {
        InfiniteGrid
        InfiniteGridSettings::default()
    }
}
