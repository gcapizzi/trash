use anyhow::{anyhow, Error};
use std::{path::Path, time::SystemTime};

pub trait Environment {
    fn var(&self, name: &str) -> Result<String, Error>;
}

pub trait FileSystem {
    fn rename<S: AsRef<Path>, D: AsRef<Path>>(
        &self,
        source: S,
        destination: D,
    ) -> Result<(), Error>;
    fn create_text_file<P: AsRef<Path>>(&self, path: P, contents: String) -> Result<(), Error>;
}

pub struct Trash<'a, E, F> {
    environment: &'a E,
    filesystem: &'a F,
}

impl<'a, E: Environment, F: FileSystem> Trash<'a, E, F> {
    pub fn new(environment: &'a E, filesystem: &'a F) -> Trash<'a, E, F> {
        Trash {
            environment,
            filesystem,
        }
    }

    pub fn put<T: AsRef<Path>>(&self, target: T, time: SystemTime) -> Result<(), Error> {
        let xdg_data_home = self.environment.var("XDG_DATA_HOME")?;
        let trash_path = Path::new(&xdg_data_home).join("Trash");
        let target_file_name = target
            .as_ref()
            .file_name()
            .ok_or(anyhow!("target {:?} has no file name"))?;

        let date_time: chrono::DateTime<chrono::Utc> = time.into();
        self.filesystem.create_text_file(
            &trash_path
                .join("info")
                .join(&target_file_name)
                .with_extension("trashinfo"),
            String::from(format!(
                "[Trash Info]\nPath={}\nDeletionDate={}",
                target.as_ref().to_string_lossy(),
                date_time.format("%+"),
            )),
        )?;

        self.filesystem
            .rename(&target, &trash_path.join("files").join(&target_file_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{offset::TimeZone, Utc};
    use expect::{
        expect,
        matchers::{equal, result::be_ok},
    };
    use std::{
        cell::RefCell,
        collections::HashMap,
        path::{Path, PathBuf},
    };

    #[test]
    fn it_moves_the_target_to_the_trash_dir() {
        let now = Utc.ymd(2004, 8, 31).and_hms(22, 32, 8).into();

        let mut env_vars = HashMap::new();
        env_vars.insert("XDG_DATA_HOME", "/xdg-data-dir");
        let environment = FakeEnvironment::new(env_vars);
        let filesystem = FakeFileSystem::new();
        let trash = Trash::new(&environment, &filesystem);

        let result = trash.put("/path/to/foo", now);

        expect(&result).to(be_ok());
        expect(&filesystem.get_rename("/path/to/foo"))
            .to(equal(Path::new("/xdg-data-dir/Trash/files/foo")));
        expect(&filesystem.get_file("/xdg-data-dir/Trash/info/foo.trashinfo")).to(equal(
            String::from("[Trash Info]\nPath=/path/to/foo\nDeletionDate=2004-08-31T22:32:08+00:00"),
        ))
    }

    struct FakeFileSystem {
        moves: RefCell<HashMap<PathBuf, PathBuf>>,
        files: RefCell<HashMap<PathBuf, String>>,
    }

    impl FakeFileSystem {
        fn new() -> FakeFileSystem {
            FakeFileSystem {
                moves: RefCell::new(HashMap::new()),
                files: RefCell::new(HashMap::new()),
            }
        }

        fn get_rename<S: AsRef<Path>>(&self, source: S) -> PathBuf {
            self.moves.borrow()[source.as_ref()].clone()
        }

        fn get_file<P: AsRef<Path>>(&self, path: P) -> String {
            self.files.borrow()[path.as_ref()].clone()
        }
    }

    impl FileSystem for FakeFileSystem {
        fn rename<S: AsRef<Path>, D: AsRef<Path>>(
            &self,
            source: S,
            destination: D,
        ) -> Result<(), Error> {
            self.moves.borrow_mut().insert(
                source.as_ref().to_path_buf(),
                destination.as_ref().to_path_buf(),
            );
            Ok(())
        }

        fn create_text_file<P: AsRef<Path>>(&self, path: P, contents: String) -> Result<(), Error> {
            self.files
                .borrow_mut()
                .insert(path.as_ref().to_path_buf(), contents);
            Ok(())
        }
    }

    struct FakeEnvironment<'a> {
        vars: HashMap<&'a str, &'a str>,
    }

    impl FakeEnvironment<'_> {
        fn new<'a>(vars: HashMap<&'a str, &'a str>) -> FakeEnvironment<'a> {
            FakeEnvironment { vars }
        }
    }

    impl Environment for FakeEnvironment<'_> {
        fn var(&self, name: &str) -> Result<String, Error> {
            Ok(self.vars[name].to_string())
        }
    }
}
