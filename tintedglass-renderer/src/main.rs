use bevy::{
    asset::{AssetMetaCheck, UnapprovedPathMode},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use tintedglass_renderer::{TintedGlassPlugins, plugins::schematic::SchematicHandle};

fn main() -> Result<()> {
    App::new()
        .add_plugins((
            bevy_web_file_drop::WebFileDropPlugin, // registers a new asset source, place before AssetPlugin
            DefaultPlugins.set(AssetPlugin {
                unapproved_path_mode: UnapprovedPathMode::Allow,
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            TintedGlassPlugins,
        ))
        .add_systems(Update, drag_and_drop)
        .run();

    Ok(())
}

fn drag_and_drop(
    mut events: MessageReader<FileDragAndDrop>,
    mut schematic_handle: ResMut<SchematicHandle>,
    asset_server: Res<AssetServer>,
) -> Result<()> {
    let Some(event) = events.read().last() else {
        return Ok(());
    };

    if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
        let path = cfg_select! {
            target_family = "wasm" => String::from(path_buf.to_str().unwrap()),
            _ => bevy::asset::AssetPath::from_path(path_buf.as_path()),
        };

        info!("Loading schematic: {}", path);
        schematic_handle.0 = asset_server.load(path);
    }

    Ok(())
}
