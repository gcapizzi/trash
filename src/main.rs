mod trash;

fn main() {}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Error};
    use expect::{expect, matchers::*};
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn it_moves_the_target_to_the_trash_dir() -> Result<(), Error> {
        let base_dir = create_tmp_dir()?;
        let file_path = base_dir.join("delete_me");
        create_text_file(&file_path, "DELETE_ME")?;
        let xdg_data_path = base_dir.join("xdg_data");

        let output = Command::new("./target/debug/trash")
            .arg(&file_path)
            .env("XDG_DATA_DIR", path_to_str(&xdg_data_path)?)
            .output()?;

        expect(&output.status.success()).to(equal(true));
        expect(&file_path).not_to(exist());
        expect(&xdg_data_path.join("Trash/files/delete_me")).to(exist());

        let contents = &fs::read_to_string(xdg_data_path.join("Trash/files/delete_me"))?;
        expect(contents).to(equal("DELETE_ME"));

        fs::remove_dir(base_dir)?;
        Ok(())
    }

    fn create_tmp_dir() -> Result<PathBuf, Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let tmp_dir = std::env::temp_dir().join(format!("trash_test_{}", now));
        fs::create_dir_all(&tmp_dir)?;
        Ok(tmp_dir)
    }

    fn create_text_file(path: &Path, contents: &str) -> Result<(), Error> {
        let mut file = fs::File::create(path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    fn path_to_str(path: &Path) -> Result<&str, Error> {
        path.to_str().ok_or(anyhow!("path is not unicode"))
    }
}
