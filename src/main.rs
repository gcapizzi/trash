mod environment;
mod filesystem;
mod trash;

use anyhow::{anyhow, Error};
use clap::{App, Arg};
use std::path::Path;

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

    trash.put(file_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use expect::{expect, matchers::*};
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn it_moves_the_target_to_the_trash_dir() {
        let base_dir = create_tmp_dir();
        let file_path = base_dir.join("delete_me");
        create_text_file(&file_path, "DELETE_ME");
        let xdg_data_path = base_dir.join("xdg_data");
        let trash_path = xdg_data_path.join("Trash");

        let output = Command::new("./target/debug/trash")
            .arg(&file_path)
            .env("XDG_DATA_DIR", &xdg_data_path.to_str().unwrap())
            .output()
            .unwrap();

        dbg!(std::str::from_utf8(&output.stderr).unwrap());

        expect(&output.status.success()).to(equal(true));
        expect(&file_path).not_to(exist());
        expect(&trash_path.join("files/delete_me")).to(exist());
        expect(&read_text_file(trash_path.join("files/delete_me"))).to(equal("DELETE_ME"));
        // expect(&xdg_data_path.join("Trash/info/delete_me.trashinfo")).to(exist());
        // let info_contents = &fs::read_to_string(xdg_data_path.join("Trash/info/delete_me"))?;
        // expect(info).to(equal("[Trash Info]\nPath={}\nDeletionDate={}"));

        fs::remove_dir_all(base_dir).unwrap();
    }

    fn create_tmp_dir() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = std::env::temp_dir().join(format!("trash_test_{}", now));
        fs::create_dir_all(&tmp_dir).unwrap();
        tmp_dir
    }

    fn create_text_file(path: &Path, contents: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    fn read_text_file<P: AsRef<Path>>(path: P) -> String {
        fs::read_to_string(path).unwrap()
    }
}
