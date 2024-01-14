use std::collections::BTreeMap;
use std::io::Error;
use std::path::Path;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::Display;

use crate::data::collections::Collection;
use crate::data::environment_function::EnvFunction;
use crate::persistence::{Persistence, PersistenceItem};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Environment {
    persistence: Persistence,
    data: BTreeMap<String, EnvironmentConfig>,
    status: EnvironmentStatus,
}

impl Environment {
    pub fn select(&self) -> Option<String> {
        self.status.select.clone()
    }
    pub fn set_select(&mut self, select: Option<String>) {
        self.status.select = select;
        self.persistence.save(
            Path::new("environment").to_path_buf(),
            "status".to_string(),
            &self.status,
        );
    }
    pub(crate) fn get_variable_hash_map(
        &self,
        collection: Option<Collection>,
    ) -> BTreeMap<String, EnvironmentItemValue> {
        self.status.select.clone().map_or_else(
            || {
                let mut result = BTreeMap::default();
                self.get(ENVIRONMENT_GLOBALS.to_string()).map(|e| {
                    for et in e.items.iter().filter(|i| i.enable) {
                        result.insert(
                            et.key.clone(),
                            EnvironmentItemValue {
                                value: et.value.clone(),
                                scope: ENVIRONMENT_GLOBALS.to_string(),
                                value_type: et.value_type.clone(),
                            },
                        );
                    }
                });
                for ef in EnvFunction::iter() {
                    result.insert(
                        "$".to_string() + ef.to_string().as_str(),
                        EnvironmentItemValue {
                            value: ef.to_string(),
                            scope: "Global".to_string(),
                            value_type: EnvironmentValueType::Function,
                        },
                    );
                }
                result
            },
            |s| {
                let mut result = BTreeMap::default();
                self.get(ENVIRONMENT_GLOBALS.to_string()).map(|e| {
                    for et in e.items.iter().filter(|i| i.enable) {
                        result.insert(
                            et.key.clone(),
                            EnvironmentItemValue {
                                value: et.value.clone(),
                                scope: ENVIRONMENT_GLOBALS.to_string(),
                                value_type: et.value_type.clone(),
                            },
                        );
                    }
                });
                self.get(s.clone()).map(|e| {
                    for et in e.items.iter().filter(|i| i.enable) {
                        result.insert(
                            et.key.clone(),
                            EnvironmentItemValue {
                                value: et.value.clone(),
                                scope: s.clone(),
                                value_type: et.value_type.clone(),
                            },
                        );
                    }
                });
                collection.map(|c| {
                    for et in c.envs.items.iter().filter(|item| item.enable) {
                        result.insert(
                            et.key.clone(),
                            EnvironmentItemValue {
                                value: et.value.clone(),
                                scope: c.folder.borrow().name.clone() + " Collection",
                                value_type: et.value_type.clone(),
                            },
                        );
                    }
                });
                for ef in EnvFunction::iter() {
                    result.insert(
                        "$".to_string() + ef.to_string().as_str(),
                        EnvironmentItemValue {
                            value: ef.to_string(),
                            scope: "Global".to_string(),
                            value_type: EnvironmentValueType::Function,
                        },
                    );
                }
                result
            },
        )
    }

    pub fn load_all(&mut self, workspace: String) -> Result<(), Error> {
        self.persistence.set_workspace(workspace);
        for key in self
            .persistence
            .load_list(Path::new("environment/data").to_path_buf())
            .iter()
        {
            if let Some(key_os) = key.file_name() {
                if let Some(key_name) = key_os.to_str() {
                    if let Some(environment_config) = self.persistence.load(key.clone()) {
                        self.data.insert(
                            Persistence::decode_with_file_name(key_name.to_string()),
                            environment_config,
                        );
                    }
                }
            }
        }
        let status = self
            .persistence
            .load(Path::new("environment/status.json").to_path_buf());
        status.map(|s| {
            self.status = s;
        });
        Ok(())
    }
    pub fn get(&self, key: String) -> Option<EnvironmentConfig> {
        self.data.get(key.as_str()).cloned()
    }
    pub fn get_data(&self) -> BTreeMap<String, EnvironmentConfig> {
        self.data.clone()
    }
    pub fn insert(&mut self, key: String, value: EnvironmentConfig) {
        self.data.insert(key.clone(), value.clone());
        self.persistence.save(
            Path::new("environment/data").to_path_buf(),
            key.clone(),
            &value,
        );
    }

    pub fn remove(&mut self, key: String) {
        self.data.remove(key.as_str());
        self.persistence
            .remove(Path::new("environment").to_path_buf(), key.clone());
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Display)]
pub enum EnvironmentValueType {
    String,
    Function,
}

impl Default for EnvironmentValueType {
    fn default() -> Self {
        EnvironmentValueType::String
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvironmentStatus {
    select: Option<String>,
}

pub const ENVIRONMENT_GLOBALS: &str = "__Globals__";

#[derive(Default, Clone)]
pub struct EnvironmentItemValue {
    pub value: String,
    pub scope: String,
    pub value_type: EnvironmentValueType,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvironmentConfig {
    pub items: Vec<EnvironmentItem>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvironmentItem {
    pub enable: bool,
    pub key: String,
    pub value: String,
    pub desc: String,
    pub value_type: EnvironmentValueType,
}
