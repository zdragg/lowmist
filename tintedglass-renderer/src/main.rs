use std::{fs::File, path::PathBuf};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use clap::Parser;
use snafu::ResultExt;
use tintedglass_renderer::{Schematic, TintedGlassPlugins};

#[derive(Parser)]
struct Args {
    /// Path to the schematic file
    schematic: PathBuf,
}

#[derive(Debug, snafu::Snafu)]
pub enum Error {
    #[snafu(display("Could not open schematic"))]
    OpenFile { source: std::io::Error },
    #[snafu(display("Could not parse schematic"))]
    ParseSchematic { source: tintedglass_model::Error },
}

#[snafu::report]
fn main() -> Result<(), Error> {
    let args = Args::parse();
    let file = File::open(args.schematic).context(OpenFileSnafu)?;
    let litematic = tintedglass_model::parse_litematic(file).context(ParseSchematicSnafu)?;

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(TintedGlassPlugins)
        .insert_resource(Schematic(litematic))
        .run();

    Ok(())
}
