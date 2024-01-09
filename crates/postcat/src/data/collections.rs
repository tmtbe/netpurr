use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::data::auth::{Auth, AuthType};
use crate::data::environment::{EnvironmentConfig, EnvironmentItemValue};
use crate::data::http::HttpRecord;
use crate::persistence::{Persistence, PersistenceItem};
use crate::script::script::ScriptScope;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Collections {
    persistence: Persistence,
    pub(crate) data: BTreeMap<String, Collection>,
}

impl Collections {
    pub fn load_all(&mut self, workspace: String) {
        self.persistence.set_workspace(workspace);
        for collection_file in self
            .persistence
            .load_list(Path::new("collections").to_path_buf())
            .iter()
        {
            if collection_file.is_file() {
                let mut collection = Collection::default();
                collection.load(
                    self.persistence.clone(),
                    Path::new("collections").to_path_buf(),
                    collection_file.clone(),
                );
                if collection.folder.borrow().name != "" {
                    self.data
                        .insert(collection.folder.borrow().name.clone(), collection.clone());
                }
            }
        }
    }
    pub fn insert_collection(&mut self, collection: Collection) {
        collection.folder.borrow_mut().fix_path(".".to_string());
        self.data
            .insert(collection.folder.borrow().name.clone(), collection.clone());
        collection.save(
            self.persistence.clone(),
            Path::new("collections").to_path_buf(),
        );
    }

    pub fn update_folder(&self, folder: Rc<RefCell<CollectionFolder>>) {
        folder.borrow().save(
            self.persistence.clone(),
            Path::new("collections")
                .join(folder.borrow().parent_path.as_str())
                .to_path_buf(),
        )
    }

    pub fn insert_http_record(
        &mut self,
        folder: Rc<RefCell<CollectionFolder>>,
        record: HttpRecord,
    ) {
        folder
            .borrow_mut()
            .requests
            .insert(record.name.clone(), record.clone());
        self.persistence.save(
            Path::new("collections")
                .join(folder.borrow().parent_path.as_str())
                .join(folder.borrow().name.as_str()),
            record.name.clone(),
            &record,
        );
    }

    pub fn remove_http_record(&mut self, folder: Rc<RefCell<CollectionFolder>>, name: String) {
        folder.borrow_mut().requests.remove(name.as_str());
        self.persistence.remove(
            Path::new("collections")
                .join(folder.borrow().parent_path.as_str())
                .join(folder.borrow().name.as_str()),
            name.clone(),
        )
    }

    pub fn insert_folder(
        &mut self,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        parent_folder
            .borrow_mut()
            .folders
            .insert(folder.borrow().name.clone(), folder.clone());
        folder.borrow_mut().fix_path(
            parent_folder.borrow().parent_path.clone() + "/" + parent_folder.borrow().name.as_str(),
        );
        self.update_folder(folder);
    }

    pub fn remove_collection(&mut self, collection_name: String) {
        self.data.remove(collection_name.as_str());
        self.persistence
            .remove_dir(Path::new("collections").join(collection_name.clone()));
        self.persistence.remove(
            Path::new("collections").to_path_buf(),
            collection_name + "@info",
        );
    }

    pub fn get_data(&self) -> BTreeMap<String, Collection> {
        self.data.clone()
    }

    pub fn get_pre_request_script_scope(&self, path: String) -> Option<ScriptScope> {
        let name = path.split("/").next()?;
        let collection = self.data.get(name)?;
        let scope = format!("collection:{}", collection.folder.borrow().name.clone());
        if collection.pre_request_script != "" {
            Some(ScriptScope {
                script: collection.pre_request_script.clone(),
                scope,
            })
        } else {
            None
        }
    }
    pub fn get_test_script_scope(&self, path: String) -> Option<ScriptScope> {
        let name = path.split("/").next()?;
        let collection = self.data.get(name)?;
        let scope = format!("collection:{}", collection.folder.borrow().name.clone());
        if collection.test_script != "" {
            Some(ScriptScope {
                script: collection.test_script.clone(),
                scope,
            })
        } else {
            None
        }
    }
    pub fn get_auth(&self, path: String) -> Auth {
        let (_, of) = self.get_folder_with_path(path.clone());
        let binding = path.clone();
        let paths: Vec<&str> = binding.split("/").collect();
        let mut auth;
        match of {
            None => {
                auth = Auth {
                    auth_type: Default::default(),
                    basic_username: "".to_string(),
                    basic_password: "".to_string(),
                    bearer_token: "".to_string(),
                }
            }
            Some(a) => auth = a.borrow().auth.clone(),
        };
        if paths.len() == 1 || auth.auth_type != AuthType::InheritAuthFromParent {
            auth
        } else {
            self.get_auth(paths[0..paths.len() - 1].join("/"))
        }
    }
    pub fn get_folder_with_path(
        &self,
        path: String,
    ) -> (String, Option<Rc<RefCell<CollectionFolder>>>) {
        let collection_paths: Vec<&str> = path.split("/").collect();
        let collection_name = &collection_paths[0].to_string();
        return match self.data.get(collection_name) {
            None => (collection_name.to_string(), None),
            Some(collection) => {
                let mut folder = collection.folder.clone();
                let folder_paths = &collection_paths[1..];
                for folder_name in folder_paths.iter() {
                    let binding = folder.borrow().folders.get(folder_name.to_owned()).cloned();
                    match binding {
                        None => {
                            return (collection_name.to_string(), None);
                        }
                        Some(get_folder) => {
                            folder = get_folder.clone();
                        }
                    }
                }
                (collection_name.to_string(), Some(folder))
            }
        };
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Collection {
    pub envs: EnvironmentConfig,
    pub folder: Rc<RefCell<CollectionFolder>>,
    pub pre_request_script: String,
    pub test_script: String,
}

impl Default for Collection {
    fn default() -> Self {
        Collection {
            envs: Default::default(),
            folder: Rc::new(RefCell::new(CollectionFolder {
                name: "".to_string(),
                parent_path: "".to_string(),
                desc: "".to_string(),
                auth: Auth {
                    auth_type: AuthType::NoAuth,
                    basic_username: "".to_string(),
                    basic_password: "".to_string(),
                    bearer_token: "".to_string(),
                },
                is_root: true,
                requests: Default::default(),
                folders: Default::default(),
            })),
            pre_request_script: "".to_string(),
            test_script: "".to_string(),
        }
    }
}

impl Collection {
    fn to_save_data(&self) -> SaveCollection {
        SaveCollection {
            envs: self.envs.clone(),
            pre_request_script: self.pre_request_script.clone(),
            test_script: self.test_script.clone(),
        }
    }
    pub fn build_envs(&self) -> BTreeMap<String, EnvironmentItemValue> {
        let mut result = BTreeMap::default();
        for item in self.envs.items.iter().filter(|i| i.enable) {
            result.insert(
                item.key.clone(),
                EnvironmentItemValue {
                    value: item.value.to_string(),
                    scope: self.folder.borrow().name.clone() + " Collection",
                    value_type: item.value_type.clone(),
                },
            );
        }
        result
    }
    fn save(&self, persistence: Persistence, path: PathBuf) {
        let save_data = self.to_save_data();
        persistence.save(
            path.clone(),
            self.folder.borrow().name.clone() + "@info",
            &save_data,
        );
        self.folder.borrow().save(persistence, path.clone());
    }
    fn load(&mut self, persistence: Persistence, dir_path: PathBuf, file_path: PathBuf) {
        file_path.clone().file_name().map(|file_name_os| {
            file_name_os.to_str().map(|file_name| {
                if file_name.ends_with("@info.json") {
                    file_name.split("@info").next().map(|folder_name| {
                        let collection: Option<Collection> = persistence.load(file_path);
                        collection.map(|c| {
                            self.envs = c.envs;
                            let mut folder = CollectionFolder::default();
                            folder.load(persistence, dir_path.join(folder_name));
                            self.folder = Rc::new(RefCell::new(folder));
                            self.pre_request_script = c.pre_request_script;
                            self.test_script = c.test_script;
                        });
                    });
                }
            })
        });
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectionFolder {
    pub name: String,
    pub parent_path: String,
    pub desc: String,
    pub auth: Auth,
    pub is_root: bool,
    pub requests: BTreeMap<String, HttpRecord>,
    pub folders: BTreeMap<String, Rc<RefCell<CollectionFolder>>>,
}

impl CollectionFolder {
    pub fn to_save_data(&self) -> SaveCollectionFolder {
        SaveCollectionFolder {
            name: self.name.clone(),
            parent_path: self.parent_path.clone(),
            desc: self.desc.clone(),
            auth: self.auth.clone(),
            is_root: self.is_root.clone(),
        }
    }
    pub fn load(&mut self, persistence: Persistence, path: PathBuf) {
        let collection_folder: Option<CollectionFolder> =
            persistence.load(path.join("folder@info.json").to_path_buf());
        collection_folder.map(|cf| {
            self.name = cf.name;
            self.parent_path = cf.parent_path;
            self.desc = cf.desc;
            self.auth = cf.auth;
            self.is_root = cf.is_root;
        });
        for item in persistence.load_list(path.clone()).iter() {
            if item.is_file() {
                item.to_str().map(|name| {
                    if name.ends_with(".json") && !name.ends_with("folder@info.json") {
                        let request: Option<HttpRecord> = persistence.load(item.clone());
                        request.map(|r| {
                            self.requests.insert(r.name.clone(), r);
                        });
                    }
                });
            } else if item.is_dir() {
                let mut child_folder = CollectionFolder::default();
                child_folder.load(persistence.clone(), item.clone());
                self.folders.insert(
                    child_folder.name.clone(),
                    Rc::new(RefCell::new(child_folder)),
                );
            }
        }
    }
    pub fn save(&self, persistence: Persistence, path: PathBuf) {
        let path = Path::new(&path).join(self.name.clone()).to_path_buf();
        for (name, request) in self.requests.iter() {
            persistence.save(path.clone(), name.clone(), request);
        }
        for (_, folder) in self.folders.iter() {
            folder.borrow().save(persistence.clone(), path.clone());
        }
        let save_data = self.to_save_data();
        persistence.save(path.clone(), "folder@info".to_string(), &save_data);
    }

    pub fn fix_path(&mut self, parent_path: String) {
        self.parent_path = parent_path;
        for (_, f) in self.folders.iter_mut() {
            f.borrow_mut()
                .fix_path(self.parent_path.clone() + "/" + self.name.as_str());
        }
    }

    pub fn get_path(&self) -> String {
        self.parent_path.clone() + "/" + self.name.as_str()
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveCollection {
    pub envs: EnvironmentConfig,
    pub pre_request_script: String,
    pub test_script: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveCollectionFolder {
    pub name: String,
    pub parent_path: String,
    pub desc: String,
    pub auth: Auth,
    pub is_root: bool,
}
