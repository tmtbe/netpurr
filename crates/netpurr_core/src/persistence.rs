use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::string::ToString;

use log::error;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::APP_NAME;

pub const PERSISTENCE_EXTENSION_RAW: &str = "yaml";
pub const PERSISTENCE_EXTENSION: &str = ".yaml";

pub fn get_persistence_path(path: &str) -> String {
    return format!("{}{}", path, PERSISTENCE_EXTENSION);
}
pub trait PersistenceItem {
    fn save<T: Serialize>(&self, path: PathBuf, key: String, data: &T);

    fn load<T: DeserializeOwned + std::fmt::Debug>(&self, path: PathBuf) -> Option<T>;
    fn load_list(&self, path: PathBuf) -> Vec<PathBuf>;
    fn remove(&self, path: PathBuf, key: String);
    fn rename(&self, old_path: PathBuf, new_path: PathBuf);
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
            root: dirs::home_dir().expect("find home dir error"),
            workspace: "default".to_string(),
        }
    }
}

impl Persistence {
    pub fn encode(key: String) -> String {
        key.as_str()
            .replace(".", "%dot")
            .trim_start_matches("/")
            .to_string()
    }
    pub fn decode(key: String) -> String {
        key.as_str().replace("%dot", ".")
    }

    pub fn decode_with_file_name(key: String) -> String {
        Persistence::decode(key.trim_end_matches(PERSISTENCE_EXTENSION).to_string())
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
        match serde_yaml::to_string(data) {
            Ok(yaml_str) => {
                let mut yaml_path = rel_path.join(save_key);
                yaml_path.set_extension(PERSISTENCE_EXTENSION_RAW);
                yaml_path.parent().map(|s| {
                    if fs::create_dir_all(s.clone()).is_err() {
                        error!("create_dir_all {:?} failed", rel_path);
                    }
                });
                let mut file_result = File::create(yaml_path.clone());
                match file_result {
                    Ok(mut file) => {
                        if file.write_all(yaml_str.as_bytes()).is_err() {
                            error!("write_all failed");
                        }
                    }
                    Err(_) => {
                        error!("create_file {:?} failed", yaml_path);
                    }
                }
            }
            Err(_) => {
                error!("serde_yaml::to_string failed");
            }
        }
    }
    fn load<T: DeserializeOwned + std::fmt::Debug>(&self, path: PathBuf) -> Option<T> {
        let workspace_dir = self.get_workspace_dir();
        let mut rel_path = path.clone();
        if !rel_path.starts_with(workspace_dir.clone()) {
            rel_path = workspace_dir.join(rel_path.clone());
        }
        match File::open(rel_path.clone()) {
            Ok(mut file) => {
                let mut content = String::new();
                match file.read_to_string(&mut content) {
                    Ok(_) => {
                        let result: serde_yaml::Result<T> = serde_yaml::from_str(content.as_str());
                        match result {
                            Ok(t) => Some(t),
                            Err(_) => {
                                error!("load {:?} failed: {:?}", path, result.unwrap_err());
                                None
                            }
                        }
                    }
                    Err(_) => {
                        error!("read_to_string {:?} failed", rel_path);
                        None
                    }
                }
            }
            Err(_) => {
                error!("open {:?} failed", rel_path);
                None
            }
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
        json_path.set_extension(PERSISTENCE_EXTENSION_RAW);
        if fs::remove_file(json_path.clone()).is_err() {
            error!("remove_file {:?} failed", json_path)
        }
    }

    fn rename(&self, old_path: PathBuf, new_path: PathBuf) {
        let workspace_dir = self.get_workspace_dir();
        let mut rel_old_path = old_path.clone();
        if !rel_old_path.starts_with(workspace_dir.clone()) {
            rel_old_path = workspace_dir.join(rel_old_path.clone());
        }
        let mut rel_new_path = new_path.clone();
        if !rel_new_path.starts_with(workspace_dir.clone()) {
            rel_new_path = workspace_dir.join(rel_new_path.clone());
        }
        if fs::rename(rel_old_path.clone(), rel_new_path.clone()).is_err() {
            error!("rename {:?} to {:?} failed", rel_old_path, rel_new_path);
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
            .join(APP_NAME)
            .join("workspaces")
            .join(self.workspace.clone());
    }

    fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }
}
