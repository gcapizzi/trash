use anyhow::{anyhow, Error};
use std::path::Path;

pub trait Environment {
    fn var(&self, name: &str) -> Result<String, Error>;
}

pub trait FileSystem {
    fn rename<S: AsRef<Path>, D: AsRef<Path>>(
        &self,
        source: S,
        destination: D,
    ) -> Result<(), Error>;
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

    pub fn put<T: AsRef<Path>>(&self, target: T) -> Result<(), Error> {
        let xdg_data_dir = self.environment.var("XDG_DATA_DIR")?;
        let target_file_name = target
            .as_ref()
            .file_name()
            .ok_or(anyhow!("target {:?} has no file name"))?;
        self.filesystem.rename(
            &target,
            Path::new(&xdg_data_dir)
                .join("Trash/files")
                .join(&target_file_name),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect::{expect, matchers::*};
    use std::{
        cell::RefCell,
        collections::HashMap,
        path::{Path, PathBuf},
    };

    #[test]
    fn it_moves_the_target_to_the_trash_dir() {
        let mut env_vars = HashMap::new();
        env_vars.insert("XDG_DATA_DIR", "/xdg-data-dir");
        let environment = FakeEnvironment::new(env_vars);
        let filesystem = FakeFileSystem::new();
        let trash = Trash::new(&environment, &filesystem);

        expect(&trash.put("/path/to/foo")).to(be_ok());
        expect(&filesystem.get_rename("/path/to/foo"))
            .to(equal(Path::new("/xdg-data-dir/Trash/files/foo")))
    }

    struct FakeFileSystem {
        moves: RefCell<HashMap<PathBuf, PathBuf>>,
    }

    impl FakeFileSystem {
        fn new() -> FakeFileSystem {
            FakeFileSystem {
                moves: RefCell::new(HashMap::new()),
            }
        }

        fn get_rename<S: AsRef<Path>>(&self, source: S) -> PathBuf {
            self.moves.borrow()[source.as_ref()].clone()
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
