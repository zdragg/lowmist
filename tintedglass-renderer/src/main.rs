mod error;
pub use error::{Error, Result};

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BlockPlugin)
        .run();
}

#[derive(Component)]
struct Block;

#[derive(Component)]
struct PaletteIndex(usize);

#[derive(Component)]
struct Position(IVec3);

#[derive(Resource)]
struct PrintTimer(Timer);

struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PrintTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(Startup, add_blocks)
            .add_systems(Update, print_blocks);
    }
}

fn add_blocks(mut commands: Commands) {
    commands.spawn((Block, PaletteIndex(0), Position(ivec3(0, 0, 0))));
    commands.spawn((Block, PaletteIndex(1), Position(ivec3(0, 0, 1))));
    commands.spawn((Block, PaletteIndex(2), Position(ivec3(2, 0, 1))));
}

fn print_blocks(
    time: Res<Time>,
    mut timer: ResMut<PrintTimer>,
    query: Query<(&PaletteIndex, &Position), With<Block>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (palette, position) in query {
            println!(
                "At position {}, there exists block with palette index {}",
                position.0, palette.0
            )
        }
    }
}
