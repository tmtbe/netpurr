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
    fn load_list(&self, path: PathBuf) -> Result<Vec<PathBuf>, Error>;
    fn remove(&self, path: PathBuf, key: String) -> Result<(), Error>;
    fn remove_dir(&self, path: PathBuf) -> Result<(), Error>;
    fn get_workspace_dir(&self) -> Result<PathBuf, Error>;
    fn set_workspace(&mut self, workspace: String);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Persistence {
    root: Option<PathBuf>,
    workspace: String,
}

impl Default for Persistence {
    fn default() -> Self {
        if let Some(home_dir) = dirs::home_dir() {
            Persistence {
                root: Some(home_dir),
                workspace: "default".to_string(),
            }
        } else {
            Persistence {
                root: None,
                workspace: "default".to_string(),
            }
        }
    }
}

impl Persistence {
    pub fn encode(key: String) -> String {
        key.as_str().replace(".", "%dot")
    }
    pub fn decode(key: String) -> String {
        key.as_str().replace("%dot", ".")
    }

    pub fn decode_with_file_name(key: String) -> String {
        Persistence::decode(key.trim_end_matches(".json").to_string())
    }
}
impl PersistenceItem for Persistence {
    fn save<T: Serialize>(&self, path: PathBuf, key: String, data: &T) -> Result<(), Error> {
        let save_key = Persistence::encode(key);
        let workspace_dir = self.get_workspace_dir()?;
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        fs::create_dir_all(rel_path.clone())?;
        let json = serde_json::to_string(data)?;
        let mut json_path = rel_path.join(save_key);
        json_path.set_extension("json");
        let mut file = File::create(json_path.clone())?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    fn load<T: DeserializeOwned>(&self, path: PathBuf) -> Result<T, Error> {
        let workspace_dir = self.get_workspace_dir()?;
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        let mut file = File::open(rel_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let result: serde_json::Result<T> = serde_json::from_str(content.as_str());
        if result.is_ok() {
            Ok(result.unwrap())
        } else {
            Err(Error::new(ErrorKind::Unsupported, "load failed"))
        }
    }
    fn load_list(&self, path: PathBuf) -> Result<Vec<PathBuf>, Error> {
        let mut result = vec![];
        let workspace_dir = self.get_workspace_dir()?;
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        let dir_path = workspace_dir.join(path);
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    result.push(entry.path().to_path_buf());
                }
            }
        }
        Ok(result)
    }
    fn remove(&self, path: PathBuf, key: String) -> Result<(), Error> {
        let save_key = Persistence::encode(key);
        let workspace_dir = self.get_workspace_dir()?;
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        let mut json_path = rel_path.join(save_key);
        json_path.set_extension("json");
        fs::remove_file(json_path);
        Ok(())
    }
    fn remove_dir(&self, path: PathBuf) -> Result<(), Error> {
        let workspace_dir = self.get_workspace_dir()?;
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        fs::remove_dir_all(rel_path);
        Ok(())
    }

    fn get_workspace_dir(&self) -> Result<PathBuf, Error> {
        if let Some(home_path) = self.root.clone() {
            let workspace_path = home_path
                .join("Poster")
                .join("workspaces")
                .join(self.workspace.clone());
            Ok(workspace_path)
        } else {
            Err(Error::new(
                ErrorKind::Unsupported,
                "workspace can not create",
            ))
        }
    }

    fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }
}
