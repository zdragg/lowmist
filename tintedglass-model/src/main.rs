use tintedglass_model;

use std::{fs::File, path::PathBuf};

use clap::Parser;
use snafu::ResultExt;

#[derive(clap::Parser)]
struct Args {
    /// Path to the schematic file
    schematic: PathBuf,
}

#[derive(Debug, snafu::Snafu)]
pub enum Error {
    #[snafu(display("Could not open {}", path.display()))]
    OpenFileError {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("Could not parse {} as a schematic", path.display()))]
    ParseSchematicError {
        source: tintedglass_model::ParseSchematicError,
        path: PathBuf,
    },
}

#[snafu::report]
fn main() -> Result<(), Error> {
    let pathbuf = Args::parse().schematic;
    let path = pathbuf.as_path();

    let file = File::open(path).context(OpenFileSnafu { path })?;

    let schematic =
        tintedglass_model::parse_schematic(file).context(ParseSchematicSnafu { path })?;

    println!("Name: {:?}", schematic.metadata.name);
    println!("Author: {:?}", schematic.metadata.author);
    println!("Description: {:?}", schematic.metadata.description);
    println!("Time Created: {:?}", schematic.metadata.time_created);
    println!("Time Modified: {:?}", schematic.metadata.time_modified);
    println!("Enclosing Size: {:?}", schematic.metadata.enclosing_size);

    let (name, region) = schematic.regions.into_iter().next().unwrap();
    println!("Region Name: {name}");

    let non_air_count = region
        .block_state_iter()
        .filter(|(_, palette)| {
            palette.name != "minecraft:air"
                && palette.name != "minecraft:void_air"
                && palette.name != "minecraft:cave_air"
        })
        .count();

    println!("these two values should be equal: ");
    println!("total non air blocks: {non_air_count}");
    println!("Metadata.TotalBlocks: {}", schematic.metadata.total_blocks);

    // Get a random block
    let slot: i32 = rand::random_range(0..schematic.metadata.total_volume);
    let slot: u32 = bytemuck::cast(slot);
    let (coord, block) = region.block_state_iter().nth(slot as usize).unwrap();
    println!("at {coord}, there is {block:?}");
    Ok(())
}
