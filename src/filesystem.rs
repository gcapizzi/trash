use crate::trash;

use anyhow::Error;
use std::{fs, io::Write, path::Path};

pub struct FileSystem {}

impl FileSystem {
    pub fn new() -> FileSystem {
        FileSystem {}
    }

    fn create_parent_dir<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        if let Some(parent) = &path.as_ref().parent() {
            fs::create_dir_all(&parent)?;
        }
        Ok(())
    }
}

impl trash::FileSystem for FileSystem {
    fn rename<S: AsRef<Path>, D: AsRef<Path>>(
        &self,
        source: S,
        destination: D,
    ) -> Result<(), Error> {
        self.create_parent_dir(&destination)?;
        fs::rename(source, destination)?;
        Ok(())
    }

    fn create_text_file<P: AsRef<Path>>(&self, path: P, contents: String) -> Result<(), Error> {
        self.create_parent_dir(&path)?;
        let mut file = fs::File::create(path).unwrap();
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::trash::FileSystem;
    use expect::{expect, matchers::*};
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn it_renames_a_files_creating_destination_dir_if_necessary() {
        let filesystem = super::FileSystem::new();

        let base_dir = create_tmp_dir();
        let source_path = &base_dir.join("foo");
        create_text_file(&source_path, "FOO");
        let destination_path = &base_dir.join("path/to/bar");

        let result = filesystem.rename(&source_path, &destination_path);

        expect(&result).to(be_ok());
        expect(&source_path).not_to(exist());
        expect(&destination_path).to(exist());
        let destination_contents = read_text_file(&destination_path);
        expect(&destination_contents).to(equal("FOO"));

        fs::remove_dir_all(base_dir).unwrap();
    }

    #[test]
    fn it_creates_a_text_file_creating_destination_dir_if_necessary() {
        let filesystem = super::FileSystem::new();

        let base_dir = create_tmp_dir();
        let file_path = &base_dir.join("path/to/file");

        let result = filesystem.create_text_file(&file_path, String::from("THE FILE CONTENTS"));

        expect(&result).to(be_ok());
        expect(&file_path).to(exist());
        let file_contents = read_text_file(&file_path);
        expect(&file_contents).to(equal(String::from("THE FILE CONTENTS")));

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

    fn create_text_file<P: AsRef<Path>>(path: P, contents: &str) {
        let mut file = fs::File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    fn read_text_file<P: AsRef<Path>>(path: P) -> String {
        fs::read_to_string(path).unwrap()
    }
}
