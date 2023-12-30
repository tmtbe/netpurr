use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use log::error;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait PersistenceItem {
    fn save<T: Serialize>(&self, path: PathBuf, key: String, data: &T);

    fn load<T: DeserializeOwned + std::fmt::Debug>(&self, path: PathBuf) -> Option<T>;
    fn load_list(&self, path: PathBuf) -> Vec<PathBuf>;
    fn remove(&self, path: PathBuf, key: String);
    fn remove_dir(&self, path: PathBuf);
    fn get_workspace_dir(&self) -> PathBuf;
    fn set_workspace(&mut self, workspace: String);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Persistence {
    root: PathBuf,
    workspace: String,
}

impl Default for Persistence {
    fn default() -> Self {
        Persistence {
            root: dirs::home_dir().unwrap(),
            workspace: "default".to_string(),
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
    fn save<T: Serialize>(&self, path: PathBuf, key: String, data: &T) {
        let save_key = Persistence::encode(key);
        let workspace_dir = self.get_workspace_dir();
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        if fs::create_dir_all(rel_path.clone()).is_err() {
            error!("create_dir_all {:?} failed", rel_path);
            return;
        };
        let json = serde_json::to_string(data);
        if json.is_err() {
            error!("serde_json::to_string failed");
            return;
        }
        let mut json_path = rel_path.join(save_key);
        json_path.set_extension("json");
        let mut file = File::create(json_path.clone());
        if file.is_err() {
            error!("create_file {:?} failed", json_path);
            return;
        }
        if file.unwrap().write_all(json.unwrap().as_bytes()).is_err() {
            error!("write_all failed");
        };
    }
    fn load<T: DeserializeOwned + std::fmt::Debug>(&self, path: PathBuf) -> Option<T> {
        let workspace_dir = self.get_workspace_dir();
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        let mut file = File::open(rel_path.clone());
        if file.is_err() {
            error!("open {:?} failed", rel_path);
            return None;
        }
        let mut content = String::new();
        if file.unwrap().read_to_string(&mut content).is_err() {
            error!("read_to_string {:?} failed", rel_path);
            return None;
        }
        let result: serde_json::Result<T> = serde_json::from_str(content.as_str());
        if result.is_ok() {
            Some(result.unwrap())
        } else {
            error!("load {:?} failed: {:?}", path, result.unwrap_err());
            None
        }
    }
    fn load_list(&self, path: PathBuf) -> Vec<PathBuf> {
        let mut result = vec![];
        let workspace_dir = self.get_workspace_dir();
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
        result
    }
    fn remove(&self, path: PathBuf, key: String) {
        let save_key = Persistence::encode(key);
        let workspace_dir = self.get_workspace_dir();
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        let mut json_path = rel_path.join(save_key);
        json_path.set_extension("json");
        if fs::remove_file(json_path.clone()).is_err() {
            error!("remove_file {:?} failed", json_path)
        }
    }
    fn remove_dir(&self, path: PathBuf) {
        let workspace_dir = self.get_workspace_dir();
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        if fs::remove_dir_all(rel_path.clone()).is_err() {
            error!("remove_dir {:?} failed", rel_path)
        }
    }

    fn get_workspace_dir(&self) -> PathBuf {
        return self
            .root
            .clone()
            .join("Poster")
            .join("workspaces")
            .join(self.workspace.clone());
    }

    fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }
}
