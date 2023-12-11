use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait PersistenceItem {
    fn save<T: Serialize>(&self, path: PathBuf, key: String, data: &T) -> Result<(), Error>;

    fn load<T: DeserializeOwned>(&self, path: PathBuf) -> Result<T, Error>;
    fn load_list(&self, path: PathBuf) -> Vec<PathBuf>;
    fn remove(&self, path: PathBuf, key: String);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Persistence {
    root: Option<PathBuf>,
}

impl Default for Persistence {
    fn default() -> Self {
        if let Some(home_dir) = dirs::home_dir() {
            Persistence {
                root: Some(home_dir),
            }
        } else {
            Persistence { root: None }
        }
    }
}

impl PersistenceItem for Persistence {
    fn save<T: Serialize>(&self, path: PathBuf, key: String, data: &T) -> Result<(), Error> {
        if let Some(home_path) = self.root.clone() {
            let dir_path = home_path.join("Poster").join(path);
            fs::create_dir_all(dir_path.clone())?;
            let json = serde_json::to_string(data)?;
            let mut json_path = dir_path.join(key.clone());
            json_path.set_extension("json");
            let mut file = File::create(json_path.clone())?;
            file.write_all(json.as_bytes())?;
        }
        Ok(())
    }

    fn remove(&self, path: PathBuf, key: String) {
        if let Some(home_path) = self.root.clone() {
            let dir_path = home_path.join("Poster").join(path);
            let mut json_path = dir_path.join(key.clone());
            json_path.set_extension("json");
            fs::remove_file(json_path);
        }
    }

    fn load<T: DeserializeOwned>(&self, path: PathBuf) -> Result<T, Error> {
        let home_path = self
            .root
            .clone()
            .ok_or(Error::new(ErrorKind::Unsupported, "load failed"))?;
        let dir_path = home_path.join("Poster").join(path);
        let mut file = File::open(dir_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let result: serde_json::Result<T> = serde_json::from_str(content.as_str());
        if result.is_ok() {
            Ok(result.unwrap())
        } else {
            Err(Error::new(ErrorKind::Unsupported, "load failed"))
        }
    }

    fn load_list(&self, path: PathBuf) -> Vec<PathBuf> {
        let mut result = vec![];
        if let Some(home_path) = self.root.clone() {
            let dir_path = home_path.join("Poster").join(path);
            if let Ok(entries) = fs::read_dir(dir_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        result.push(
                            entry
                                .path()
                                .strip_prefix(home_path.join("Poster"))
                                .unwrap()
                                .to_path_buf(),
                        );
                    }
                }
            }
        }
        result
    }
}
