use bevy::{
    asset::{AssetMetaCheck, UnapprovedPathMode, io::web::WebAssetPlugin},
    prelude::*,
};
use lowmist::{LowmistPlugins, plugins::blocks::schematic::LoadedSchematic};

fn main() {
    #[cfg(not(target_family = "wasm"))]
    start()
}

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn start() {
    App::new()
        .add_plugins((
            bevy_web_file_drop::WebFileDropPlugin, // registers a new asset source, place before AssetPlugin
            DefaultPlugins
                .set(WebAssetPlugin {
                    silence_startup_warning: true,
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            // diagnostic::FrameTimeDiagnosticsPlugin::default(),
            // diagnostic::LogDiagnosticsPlugin::default(),
            LowmistPlugins,
        ))
        .add_systems(Update, drag_and_drop)
        .run();
}

fn drag_and_drop(
    mut events: MessageReader<FileDragAndDrop>,
    mut loaded_schem: ResMut<LoadedSchematic>,
    asset_server: Res<AssetServer>,
) -> Result {
    let Some(event) = events.read().last() else {
        return Ok(());
    };

    if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
        let path = cfg_select! {
            target_family = "wasm" => String::from(path_buf.to_str().unwrap()),
            _ => bevy::asset::AssetPath::from_path(path_buf.as_path()),
        };

        info!("Loading schematic: {}", path);
        loaded_schem.0 = asset_server.load(path);
    }

    Ok(())
}
