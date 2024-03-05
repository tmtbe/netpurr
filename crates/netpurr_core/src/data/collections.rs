use openapiv3::OpenAPI;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::data::auth::{Auth, AuthType};
use crate::data::environment::{EnvironmentConfig, EnvironmentItemValue};
use crate::data::record::Record;
use crate::persistence::{Persistence, PersistenceItem};
use crate::script::ScriptScope;

#[derive(Default, Clone, Debug)]
pub struct Collections {
    persistence: Persistence,
    pub data: BTreeMap<String, Collection>,
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
    pub fn add_collection(&mut self, collection: Collection) {
        collection
            .folder
            .borrow_mut()
            .fix_path_recursion(".".to_string());
        self.data
            .insert(collection.folder.borrow().name.clone(), collection.clone());
        collection.save_recursively(
            self.persistence.clone(),
            Path::new("collections").to_path_buf(),
        );
    }

    pub fn add_record(&mut self, folder: Rc<RefCell<CollectionFolder>>, record: Record) {
        folder
            .borrow_mut()
            .requests
            .insert(record.name(), record.clone());
        self.persistence.save(
            Path::new("collections")
                .join(folder.borrow().parent_path.as_str())
                .join(folder.borrow().name.as_str()),
            record.name(),
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

    pub fn add_folder(
        &mut self,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        parent_folder
            .borrow_mut()
            .folders
            .insert(folder.borrow().name.clone(), folder.clone());
        folder.borrow_mut().fix_path_recursion(
            parent_folder.borrow().parent_path.clone() + "/" + parent_folder.borrow().name.as_str(),
        );
        folder.borrow().save_recursively(
            self.persistence.clone(),
            Path::new("collections")
                .join(folder.borrow().parent_path.as_str())
                .to_path_buf(),
        );
    }
    pub fn update_folder_info(
        &self,
        old_folder_name: String,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        let parent_path = folder.borrow().parent_path.clone();
        folder.borrow_mut().fix_path_recursion(parent_path);
        parent_folder
            .borrow_mut()
            .folders
            .remove(old_folder_name.as_str());
        let new_folder_name = folder.borrow().name.clone();
        parent_folder
            .borrow_mut()
            .folders
            .insert(new_folder_name.clone(), folder.clone());
        // rename folder
        self.persistence.rename(
            Path::new("collections")
                .join(folder.borrow().parent_path.clone())
                .join(old_folder_name.clone()),
            Path::new("collections")
                .join(folder.borrow().parent_path.clone())
                .join(new_folder_name.clone()),
        );
        // add new @info
        folder.borrow().save_info(
            self.persistence.clone(),
            Path::new("collections")
                .join(folder.borrow().parent_path.clone())
                .join(folder.borrow().name.clone())
                .to_path_buf(),
        );
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
    pub fn update_collection_info(&mut self, old_collection_name: String, collection: Collection) {
        collection
            .folder
            .borrow_mut()
            .fix_path_recursion(".".to_string());
        self.data.remove(old_collection_name.as_str());
        let new_collection_name = collection.folder.borrow().name.clone();
        self.data
            .insert(new_collection_name.clone(), collection.clone());
        // remove old @info
        self.persistence.remove(
            Path::new("collections").to_path_buf(),
            old_collection_name + "@info",
        );
        // add new @info
        collection.save_info(
            self.persistence.clone(),
            Path::new("collections").to_path_buf(),
        );
        // update new folder @info
        collection.folder.borrow().save_info(
            self.persistence.clone(),
            Path::new("collections")
                .join(collection.folder.borrow().name.clone())
                .to_path_buf(),
        );
    }

    pub fn remove_folder(&self, parent_folder: Rc<RefCell<CollectionFolder>>, name: String) {
        parent_folder.borrow_mut().folders.remove(name.as_str());
        self.persistence.remove_dir(
            Path::new("collections")
                .join(parent_folder.borrow().parent_path.as_str())
                .join(parent_folder.borrow().name.as_str())
                .join(name.as_str()),
        );
    }

    pub fn get_data(&self) -> BTreeMap<String, Collection> {
        self.data.clone()
    }

    pub fn get_path_scripts(&self, path: String) -> (Vec<ScriptScope>, Vec<ScriptScope>) {
        let mut name_builder = Vec::new();
        let mut pre_scripts = Vec::new();
        let mut test_scripts = Vec::new();
        for path_part in path.split("/") {
            name_builder.push(path_part);
            self.get_folder_with_path(name_builder.join("/"))
                .1
                .map(|f| {
                    test_scripts.push(ScriptScope {
                        script: f.borrow().test_script.clone(),
                        scope: name_builder.join("/"),
                    });
                    pre_scripts.push(ScriptScope {
                        script: f.borrow().pre_request_script.clone(),
                        scope: name_builder.join("/"),
                    });
                });
        }
        (pre_scripts, test_scripts)
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Collection {
    pub envs: EnvironmentConfig,
    pub openapi: Option<OpenAPI>,
    pub folder: Rc<RefCell<CollectionFolder>>,
}

impl Default for Collection {
    fn default() -> Self {
        Collection {
            envs: Default::default(),
            openapi: None,
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
                pre_request_script: "".to_string(),
                test_script: "".to_string(),
            })),
        }
    }
}

impl Collection {
    pub fn duplicate(&self, name: String) -> Self {
        let json = serde_json::to_string(self).unwrap();
        let dup: Self = serde_json::from_str(json.as_str()).unwrap();
        dup.folder.borrow_mut().name = name;
        dup
    }
    fn to_save_data(&self) -> SaveCollection {
        SaveCollection {
            envs: self.envs.clone(),
            openapi: self.openapi.clone(),
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
    fn save_info(&self, persistence: Persistence, path: PathBuf) {
        let save_data = self.to_save_data();
        persistence.save(
            path.clone(),
            self.folder.borrow().name.clone() + "@info",
            &save_data,
        );
    }
    fn save_recursively(&self, persistence: Persistence, path: PathBuf) {
        self.save_info(persistence.clone(), path.clone());
        self.folder
            .borrow()
            .save_recursively(persistence, path.clone());
    }
    fn load(&mut self, persistence: Persistence, dir_path: PathBuf, file_path: PathBuf) {
        file_path.clone().file_name().map(|file_name_os| {
            file_name_os.to_str().map(|file_name| {
                if file_name.ends_with("@info.json") {
                    file_name.split("@info").next().map(|folder_name| {
                        let collection: Option<Collection> = persistence.load(file_path);
                        collection.map(|c| {
                            self.envs = c.envs;
                            self.openapi = c.openapi;
                            let mut folder = CollectionFolder::default();
                            folder.load(persistence, dir_path.join(folder_name));
                            self.folder = Rc::new(RefCell::new(folder));
                        });
                    });
                }
            })
        });
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectionFolder {
    pub name: String,
    pub parent_path: String,
    pub desc: String,
    pub auth: Auth,
    pub is_root: bool,
    pub requests: BTreeMap<String, Record>,
    pub folders: BTreeMap<String, Rc<RefCell<CollectionFolder>>>,
    pub pre_request_script: String,
    pub test_script: String,
}

impl CollectionFolder {
    pub fn duplicate(&self, new_name: String) -> Self {
        let json = serde_json::to_string(self).unwrap();
        let mut dup: Self = serde_json::from_str(json.as_str()).unwrap();
        dup.name = new_name;
        dup
    }
    pub fn to_save_data(&self) -> SaveCollectionFolder {
        SaveCollectionFolder {
            desc: self.desc.clone(),
            auth: self.auth.clone(),
            is_root: self.is_root.clone(),
            pre_request_script: self.pre_request_script.clone(),
            test_script: self.test_script.clone(),
        }
    }
    pub fn load(&mut self, persistence: Persistence, path: PathBuf) {
        let collection_folder: Option<CollectionFolder> =
            persistence.load(path.join("folder@info.json").to_path_buf());
        let path_str = path.to_str().unwrap_or_default().trim_start_matches(
            persistence
                .get_workspace_dir()
                .join("collections")
                .to_str()
                .unwrap_or_default(),
        );
        let path_split: Vec<&str> = path_str.split("/").collect();
        let name = path_split[path_split.len() - 1].to_string();
        let mut parent_path = path_split[1..path_split.len() - 1].join("/");
        if parent_path.is_empty() {
            parent_path = ".".to_string();
        }
        collection_folder.map(|cf| {
            self.name = name;
            self.parent_path = parent_path;
            self.desc = cf.desc;
            self.auth = cf.auth;
            self.is_root = cf.is_root;
            self.pre_request_script = cf.pre_request_script;
            self.test_script = cf.test_script;
        });
        for item in persistence.load_list(path.clone()).iter() {
            if item.is_file() {
                item.to_str().map(|name| {
                    if name.ends_with(".json") && !name.ends_with("folder@info.json") {
                        let request: Option<Record> = persistence.load(item.clone());
                        request.map(|r| {
                            self.requests.insert(r.name(), r);
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
    pub fn save_info(&self, persistence: Persistence, path: PathBuf) {
        let save_data = self.to_save_data();
        persistence.save(path.clone(), "folder@info".to_string(), &save_data);
    }
    pub fn save_recursively(&self, persistence: Persistence, path: PathBuf) {
        let path = Path::new(&path).join(self.name.clone()).to_path_buf();
        for (name, request) in self.requests.iter() {
            persistence.save(path.clone(), name.clone(), request);
        }
        for (_, folder) in self.folders.iter() {
            folder
                .borrow()
                .save_recursively(persistence.clone(), path.clone());
        }
        self.save_info(persistence.clone(), path.clone());
    }

    pub fn fix_path_recursion(&mut self, parent_path: String) {
        self.parent_path = parent_path;
        for (_, f) in self.folders.iter_mut() {
            f.borrow_mut()
                .fix_path_recursion(self.parent_path.clone() + "/" + self.name.as_str());
        }
    }

    pub fn get_path(&self) -> String {
        self.parent_path.clone() + "/" + self.name.as_str()
    }
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveCollection {
    pub envs: EnvironmentConfig,
    pub openapi: Option<OpenAPI>,
}

#[derive(Default, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveCollectionFolder {
    pub desc: String,
    pub auth: Auth,
    pub is_root: bool,
    pub pre_request_script: String,
    pub test_script: String,
}
