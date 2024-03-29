use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::APP_NAME;
use crate::data::workspace::Workspace;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigData {
    select_workspace: Option<String>,
    select_collection: Option<String>,
    #[serde(skip, default)]
    workspaces: BTreeMap<String, Workspace>,
}

impl Default for ConfigData {
    fn default() -> Self {
        let mut workspaces = BTreeMap::default();
        workspaces.insert(
            "default".to_string(),
            Workspace {
                name: "default".to_string(),
                path: dirs::home_dir()
                    .unwrap_or(Path::new("/home").to_path_buf())
                    .join(APP_NAME)
                    .join("workspaces")
                    .join("default"),
                enable_git: None,
                remote_url: None,
            },
        );
        ConfigData {
            select_workspace: None,
            select_collection: None,
            workspaces,
        }
    }
}

impl ConfigData {
    pub fn load() -> Self {
        let mut config_data = if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(APP_NAME).join("config.json");
            match File::open(config_path) {
                Ok(mut file) => {
                    let mut content = String::new();
                    match file.read_to_string(&mut content) {
                        Ok(_) => {
                            let result: serde_json::Result<Self> =
                                serde_json::from_str(content.as_str());
                            result.unwrap_or_else(|_| Self::default())
                        }
                        Err(_) => Self::default(),
                    }
                }
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        };
        config_data.refresh_workspaces();
        config_data
    }
    pub fn new_workspace(&mut self, name: String) {
        if let Some(home_dir) = dirs::home_dir() {
            let workspaces_path = home_dir.join(APP_NAME).join("workspaces").join(name);
            fs::create_dir_all(workspaces_path);
            self.refresh_workspaces();
        }
    }
    pub fn refresh_workspaces(&mut self) {
        self.workspaces.clear();
        if let Some(home_dir) = dirs::home_dir() {
            let workspaces_path = home_dir.join(APP_NAME).join("workspaces");
            if let Ok(entries) = fs::read_dir(workspaces_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if entry.path().is_dir() {
                            if let Some(file_name) = entry.file_name().to_str() {
                                self.workspaces.insert(
                                    file_name.to_string(),
                                    Workspace {
                                        name: file_name.to_string(),
                                        path: entry.path(),
                                        enable_git: None,
                                        remote_url: None,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn select_workspace(&self) -> Option<String> {
        self.select_workspace.clone()
    }
    pub fn select_collection(&self) -> Option<String> {
        self.select_collection.clone()
    }
    pub fn set_select_workspace(&mut self, select_workspace: Option<String>) {
        self.select_workspace = select_workspace;
        self.save();
    }
    pub fn set_select_collection(&mut self, collection: Option<String>) {
        self.select_collection = collection;
        self.save();
    }
    pub fn set_select_workspace_collection(
        &mut self,
        select_workspace: Option<String>,
        collection: Option<String>,
    ) {
        self.select_workspace = select_workspace;
        self.select_collection = collection;
        self.save();
    }
    fn save(&self) -> Result<(), Error> {
        let json = serde_json::to_string(self)?;
        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(APP_NAME).join("config.json");
            let mut file = File::create(config_path.clone())?;
            file.write_all(json.as_bytes())?;
        }
        Ok(())
    }
    pub fn workspaces(&self) -> &BTreeMap<String, Workspace> {
        &self.workspaces
    }
    pub fn mut_workspaces(&mut self) -> &mut BTreeMap<String, Workspace> {
        &mut self.workspaces
    }
}
