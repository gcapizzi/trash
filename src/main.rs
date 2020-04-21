mod environment;
mod filesystem;
mod trash;

use anyhow::{anyhow, Error};
use clap::{App, Arg};
use std::{path::Path, time::SystemTime};

fn main() -> Result<(), Error> {
    let matches = App::new("trash")
        .arg(
            Arg::with_name("FILE")
                .help("The file to move to the trash")
                .required(true)
                .index(1),
        )
        .get_matches();

    let file_path_str = matches
        .value_of("FILE")
        .ok_or(anyhow!("file not specified"))?;
    let file_path = Path::new(file_path_str);

    let environment = environment::Environment::new();
    let filesystem = filesystem::FileSystem::new();
    let trash = trash::Trash::new(&environment, &filesystem);

    trash.put(file_path, SystemTime::now())?;

    Ok(())
}
