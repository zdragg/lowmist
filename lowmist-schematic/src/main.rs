use lowmist_schematic::Litematic;

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
    OpenFile {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("Could not parse {} as a schematic", path.display()))]
    ParseSchematic {
        source: lowmist_schematic::Error,
        path: PathBuf,
    },
}

#[snafu::report]
fn main() -> Result<(), Error> {
    let pathbuf = Args::parse().schematic;
    let path = pathbuf.as_path();

    let file = File::open(path).context(OpenFileSnafu { path })?;

    let schematic = Litematic::parse(file).context(ParseSchematicSnafu { path })?;

    println!(
        "Schematic version: {}.{}",
        schematic.version, schematic.sub_version
    );
    println!(
        "Minecraft Data version: {}",
        schematic.minecraft_data_version
    );

    println!("");

    println!("Name: {:?}", schematic.metadata.name);
    println!("Author: {:?}", schematic.metadata.author);
    println!("Description: {:?}", schematic.metadata.description);
    println!("Time Created: {:?}", schematic.metadata.time_created);
    println!("Time Modified: {:?}", schematic.metadata.time_modified);
    println!("Enclosing Size: {:?}", schematic.metadata.enclosing_size);

    println!("");

    println!("Region count: {}", schematic.regions.len());

    let non_air_count = schematic.blocks_without_air().count();

    println!(
        "these two values should be equal IF NO OVERLAPPING REGIONS. If overlap, then former < latter"
    );
    println!("total non air blocks counted by iterator: {non_air_count}");
    println!("Metadata.TotalBlocks: {}", schematic.metadata.total_blocks);

    println!(
        "these two values should also be equal IF NO OVERLAPPING REGIONS. If overlap, then former < latter"
    );
    let total_block_count = schematic.blocks().count();
    println!("total blocks counted by iterator: {}", total_block_count);
    println!("Metadata.TotalVolume: {}", schematic.metadata.total_volume);

    // Print the whole palette
    println!("Printing all palette entries:");
    for (id, entry) in schematic.palettes() {
        println!("At {id:?}, there is {entry:?}");
    }

    println!("");
    for (name, _region) in &schematic.regions {
        println!("There exists region: {name}");
    }
    Ok(())
}
