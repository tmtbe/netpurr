use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Local, NaiveDate, Utc};
use eframe::epaint::ahash::HashMap;
use egui::TextBuffer;
use log::info;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use rustygit::Repository;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use url::Url;
use uuid::Uuid;

use cookie_store::{CookieDomain, RawCookie};
use reqwest_cookie_store::CookieStoreMutex;

use crate::env_func::EnvFunction;
use crate::save::{Persistence, PersistenceItem};
use crate::script::script::ScriptScope;
use crate::utils;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfigData {
    select_workspace: String,
    #[serde(skip, default)]
    workspaces: BTreeMap<String, Workspace>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Workspace {
    pub name: String,
    pub path: PathBuf,
    pub enable_git: Option<bool>,
    pub remote_url: Option<String>,
}

impl Workspace {
    pub fn if_enable_git(&mut self) -> bool {
        if let Some(git) = self.enable_git {
            return git;
        }
        let repo = Repository::new(self.path.clone());
        match repo.cmd(["status"]) {
            Ok(_) => {
                self.enable_git = Some(true);
                true
            }
            Err(_) => false,
        }
    }

    pub fn has_remote_url(&mut self) -> bool {
        if let Some(url) = &self.remote_url {
            return true;
        }
        let repo = Repository::new(self.path.clone());
        if let Ok(remote) = repo.cmd_out(["remote", "get-url", "origin"]) {
            if remote.len() > 0 {
                self.remote_url = Some(remote[0].clone());
                return true;
            } else {
                return false;
            }
        }
        false
    }
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
                    .join("Poster")
                    .join("workspaces")
                    .join("default"),
                enable_git: None,
                remote_url: None,
            },
        );
        ConfigData {
            select_workspace: "default".to_string(),
            workspaces: workspaces,
        }
    }
}

impl ConfigData {
    pub fn load() -> Self {
        let mut config_data = if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join("Poster").join("config.json");
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
            let workspaces_path = home_dir.join("Poster").join("workspaces").join(name);
            fs::create_dir_all(workspaces_path);
            self.refresh_workspaces();
        }
    }
    pub fn refresh_workspaces(&mut self) {
        self.workspaces.clear();
        if let Some(home_dir) = dirs::home_dir() {
            let workspaces_path = home_dir.join("Poster").join("workspaces");
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
    pub fn select_workspace(&self) -> &str {
        &self.select_workspace
    }
    pub fn set_select_workspace(&mut self, select_workspace: String) {
        self.select_workspace = select_workspace;
        self.save();
    }

    fn save(&self) -> Result<(), Error> {
        let json = serde_json::to_string(self)?;
        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join("Poster").join("config.json");
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

#[derive(Default, Clone, Debug)]
pub struct WorkspaceData {
    pub workspace_name: String,
    pub cookies_manager: CookiesManager,
    pub central_request_data_list: CentralRequestDataList,
    pub history_data_list: HistoryDataList,
    pub environment: Environment,
    pub collections: Collections,
    client: Option<Client>,
}

impl WorkspaceData {
    pub(crate) fn build_client(&mut self) -> Client {
        match &self.client {
            None => {
                let client = Client::builder()
                    .cookie_provider(self.cookies_manager.cookie_store.clone())
                    .trust_dns(true)
                    .tcp_nodelay(true)
                    .timeout(Duration::from_secs(60))
                    .build()
                    .unwrap();
                self.client = Some(client.clone());
                client
            }
            Some(client) => client.clone(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct CookiesManager {
    persistence: Persistence,
    pub cookie_store: Arc<CookieStoreMutex>,
    pub client: Option<Client>,
}

impl CookiesManager {
    pub fn load_all(&mut self, workspace: String) {
        self.persistence.set_workspace(workspace);
        let cookie_store = {
            if let Ok(file) = File::open(self.persistence.get_workspace_dir().join("cookies.json"))
                .map(std::io::BufReader::new)
            {
                // use re-exported version of `CookieStore` for crate compatibility
                reqwest_cookie_store::CookieStore::load_json(file).unwrap()
            } else {
                reqwest_cookie_store::CookieStore::default()
            }
        };
        let cookie_store = CookieStoreMutex::new(cookie_store);
        self.cookie_store = Arc::new(cookie_store);
    }

    pub fn save(&self) {
        let mut store = self.cookie_store.lock().unwrap();
        for c in store.iter_any() {
            info!("{:?}", c);
        }
        let mut writer = File::create(self.persistence.get_workspace_dir().join("cookies.json"))
            .map(std::io::BufWriter::new)
            .unwrap();
        store
            .save_incl_expired_and_nonpersistent_json(&mut writer)
            .unwrap();
    }
    pub fn contain_domain(&self, domain: String) -> bool {
        self.cookie_store
            .lock()
            .unwrap()
            .iter_any()
            .find(|c| {
                c.domain == CookieDomain::HostOnly(domain.clone())
                    || c.domain == CookieDomain::Suffix(domain.clone())
            })
            .is_some()
    }
    pub fn contain_domain_key(&self, domain: String, key: String) -> bool {
        self.cookie_store
            .lock()
            .unwrap()
            .iter_any()
            .find(|c| c.domain() == Some(domain.as_str()) && c.name() == key.as_str())
            .is_some()
    }
    pub fn get_url_cookies(&self, mut url: String) -> BTreeMap<String, Cookie> {
        let mut result = BTreeMap::new();
        if !url.starts_with("http") {
            url = "http://".to_string() + url.as_str();
        }
        let url_parse = Url::parse(url.as_str());
        match url_parse {
            Ok(url_parse_ok) => {
                self.cookie_store
                    .lock()
                    .unwrap()
                    .matches(&url_parse_ok)
                    .iter()
                    .map(|c| Cookie {
                        name: c.name().to_string(),
                        value: c.value().to_string(),
                        domain: c.domain().unwrap_or("").to_string(),
                        path: c.path().unwrap_or("").to_string(),
                        expires: c
                            .expires_datetime()
                            .map(|e| e.to_string())
                            .unwrap_or("".to_string()),
                        max_age: c.max_age().map(|d| d.to_string()).unwrap_or("".to_string()),
                        raw: c.to_string(),
                        http_only: c.http_only().unwrap_or(false),
                        secure: c.secure().unwrap_or(false),
                    })
                    .for_each(|c| {
                        result.insert(c.name.clone(), c);
                    });
                result
            }
            Err(_) => BTreeMap::default(),
        }
    }
    pub fn get_domain_cookies(&self, domain: String) -> Option<BTreeMap<String, Cookie>> {
        let mut result = BTreeMap::new();
        self.cookie_store
            .lock()
            .unwrap()
            .iter_any()
            .filter(|c| {
                c.domain == CookieDomain::HostOnly(domain.clone())
                    || c.domain == CookieDomain::Suffix(domain.clone())
            })
            .map(|c| Cookie {
                name: c.name().to_string(),
                value: c.value().to_string(),
                domain: domain.clone(),
                path: c.path().unwrap_or("").to_string(),
                expires: c
                    .expires_datetime()
                    .map(|e| e.to_string())
                    .unwrap_or("".to_string()),
                max_age: c.max_age().map(|d| d.to_string()).unwrap_or("".to_string()),
                raw: c.to_string(),
                http_only: c.http_only().unwrap_or(false),
                secure: c.secure().unwrap_or(false),
            })
            .for_each(|c| {
                result.insert(c.name.clone(), c);
            });
        Some(result)
    }
    pub fn add_domain_cookies(&self, cookie: Cookie) -> Result<(), String> {
        let result = match &RawCookie::parse(cookie.raw.as_str()) {
            Ok(c) => match self.cookie_store.lock().unwrap().insert_raw_no_url_check(c) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        };
        if result.is_ok() {
            self.save();
        }
        result
    }
    pub fn update_domain_cookies(
        &self,
        cookie: Cookie,
        domain: String,
        name: String,
    ) -> Result<(), String> {
        let result = match &RawCookie::parse(cookie.raw.as_str()) {
            Ok(c) => {
                if c.name() != name || (c.domain().is_some() && c.domain() != Some(domain.as_str()))
                {
                    return Err("Domain or Name is changed.".to_string());
                }
                match self.cookie_store.lock().unwrap().insert_raw_no_url_check(c) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        };
        if result.is_ok() {
            self.save();
        }
        result
    }

    pub fn remove_domain(&mut self, mut domain: String) {
        self.cookie_store
            .lock()
            .unwrap()
            .remove_domain(domain.as_str());
        self.save()
    }
    pub fn remove_domain_path_name(&mut self, mut domain: String, path: String, name: String) {
        self.cookie_store
            .lock()
            .unwrap()
            .remove(domain.as_str(), path.as_str(), name.as_str());
        self.save()
    }
    pub fn get_cookie_domains(&self) -> Vec<String> {
        let mut hash_set: BTreeSet<String> = BTreeSet::new();
        let mut store = self.cookie_store.lock().unwrap();
        for cookie in store.iter_any() {
            match &cookie.domain {
                CookieDomain::HostOnly(domain) => {
                    hash_set.insert(domain.clone());
                }
                CookieDomain::Suffix(domain) => {
                    hash_set.insert(domain.clone());
                }
                CookieDomain::NotPresent => {}
                CookieDomain::Empty => {}
            }
        }
        let mut keys = vec![];
        for domain_name in hash_set {
            keys.push(domain_name)
        }
        keys
    }
}

impl WorkspaceData {
    pub fn load_all(&mut self, workspace: String) {
        self.central_request_data_list.load_all(workspace.clone());
        self.history_data_list.load_all(workspace.clone());
        self.environment.load_all(workspace.clone());
        self.collections.load_all(workspace.clone());
        self.cookies_manager.load_all(workspace.clone())
    }
    pub fn reload_data(&mut self, workspace: String) {
        self.history_data_list.load_all(workspace.clone());
        self.environment.load_all(workspace.clone());
        self.collections.load_all(workspace.clone());
        self.cookies_manager.load_all(workspace.clone())
    }

    pub fn get_collection(&self, option_path: Option<String>) -> Option<Collection> {
        let path = option_path?;
        let collection_name = path.splitn(2, "/").next()?;
        self.collections.data.get(collection_name).cloned()
    }
    fn get_variable_hash_map(
        &self,
        collection_path: Option<String>,
    ) -> BTreeMap<String, EnvironmentItemValue> {
        self.environment
            .get_variable_hash_map(self.get_collection(collection_path))
    }

    pub fn get_mut_crt_and_envs_auth(
        &mut self,
        id: String,
    ) -> (
        &mut CentralRequestItem,
        BTreeMap<String, EnvironmentItemValue>,
        Auth,
    ) {
        let data = self
            .central_request_data_list
            .data_map
            .get(id.as_str())
            .unwrap();
        let envs = self.get_variable_hash_map(data.collection_path.clone());

        let mut auth;
        match &data.collection_path {
            None => {
                auth = Auth {
                    auth_type: AuthType::NoAuth,
                    basic_username: "".to_string(),
                    basic_password: "".to_string(),
                    bearer_token: "".to_string(),
                }
            }
            Some(collection_path) => {
                auth = self.collections.get_auth(collection_path.clone());
            }
        }
        (
            self.central_request_data_list
                .data_map
                .get_mut(id.as_str())
                .unwrap(),
            envs,
            auth,
        )
    }

    pub fn get_mut_crt_and_envs_auth_script(
        &mut self,
        id: String,
    ) -> (
        &mut CentralRequestItem,
        BTreeMap<String, EnvironmentItemValue>,
        Auth,
        Option<ScriptScope>,
        Option<ScriptScope>,
    ) {
        let data = self
            .central_request_data_list
            .data_map
            .get(id.as_str())
            .unwrap();
        let envs = self.get_variable_hash_map(data.collection_path.clone());

        let mut auth;
        let mut pre_request_script_scope = None;
        let mut test_script_scope = None;
        match &data.collection_path {
            None => {
                auth = Auth {
                    auth_type: AuthType::NoAuth,
                    basic_username: "".to_string(),
                    basic_password: "".to_string(),
                    bearer_token: "".to_string(),
                }
            }
            Some(collection_path) => {
                auth = self.collections.get_auth(collection_path.clone());
                pre_request_script_scope = self
                    .collections
                    .get_pre_request_script_scope(collection_path.clone());
                test_script_scope = self
                    .collections
                    .get_test_script_scope(collection_path.clone());
            }
        }
        (
            self.central_request_data_list
                .data_map
                .get_mut(id.as_str())
                .unwrap(),
            envs,
            auth,
            pre_request_script_scope,
            test_script_scope,
        )
    }

    pub fn get_crt_and_envs_auth(
        &self,
        id: String,
    ) -> (
        CentralRequestItem,
        BTreeMap<String, EnvironmentItemValue>,
        Auth,
    ) {
        let data = self
            .central_request_data_list
            .data_map
            .get(id.as_str())
            .unwrap();
        let envs = self.get_variable_hash_map(data.collection_path.clone());
        let mut auth;
        match &data.collection_path {
            None => {
                auth = Auth {
                    auth_type: AuthType::NoAuth,
                    basic_username: "".to_string(),
                    basic_password: "".to_string(),
                    bearer_token: "".to_string(),
                }
            }
            Some(collection_path) => {
                auth = self.collections.get_auth(collection_path.clone());
            }
        }
        (data.clone(), envs, auth)
    }

    pub fn get_crt_and_envs_auth_script(
        &self,
        id: String,
    ) -> (
        CentralRequestItem,
        BTreeMap<String, EnvironmentItemValue>,
        Auth,
        Option<ScriptScope>,
    ) {
        let data = self
            .central_request_data_list
            .data_map
            .get(id.as_str())
            .unwrap();
        let envs = self.get_variable_hash_map(data.collection_path.clone());
        let mut auth;
        let mut script_scope = None;
        match &data.collection_path {
            None => {
                auth = Auth {
                    auth_type: AuthType::NoAuth,
                    basic_username: "".to_string(),
                    basic_password: "".to_string(),
                    bearer_token: "".to_string(),
                }
            }
            Some(collection_path) => {
                auth = self.collections.get_auth(collection_path.clone());
                script_scope = self
                    .collections
                    .get_pre_request_script_scope(collection_path.clone());
            }
        }
        (data.clone(), envs, auth, script_scope)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Collections {
    persistence: Persistence,
    data: BTreeMap<String, Collection>,
}

impl Collections {
    fn load_all(&mut self, workspace: String) {
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

    pub fn get_mut_folder_with_path(
        &mut self,
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
                    let get = folder.borrow().folders.get(folder_name.to_owned()).cloned();
                    if get.is_none() {
                        return (collection_name.to_string(), None);
                    } else {
                        folder = get.unwrap().clone();
                    }
                }
                (collection_name.to_string(), Some(folder))
            }
        };
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
                    let get = folder.borrow().folders.get(folder_name.to_owned()).cloned();
                    if get.is_none() {
                        return (collection_name.to_string(), None);
                    } else {
                        folder = get.unwrap().clone();
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

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveCollection {
    pub envs: EnvironmentConfig,
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

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SaveCollectionFolder {
    pub name: String,
    pub parent_path: String,
    pub desc: String,
    pub auth: Auth,
    pub is_root: bool,
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

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Environment {
    persistence: Persistence,
    data: BTreeMap<String, EnvironmentConfig>,
    status: EnvironmentStatus,
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
    fn get_variable_hash_map(
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
    pub value_type: EnvironmentValueType,
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

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestDataList {
    pub select_id: Option<String>,
    pub data_list: Vec<String>,
    pub data_map: HashMap<String, CentralRequestItem>,
    persistence: Persistence,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(default)]
struct CentralRequestDataListSaved {
    pub select_id: Option<String>,
    pub data_map: HashMap<String, CentralRequestItem>,
}

impl CentralRequestDataList {
    pub fn load_all(&mut self, workspace: String) {
        self.persistence.set_workspace(workspace);
        let result: Option<CentralRequestDataListSaved> = self
            .persistence
            .load(Path::new("requests/data.json").to_path_buf());
        match result {
            Some(mut c) => {
                match &c.select_id {
                    None => {}
                    Some(id) => {
                        if !c.data_map.contains_key(id.as_str()) {
                            c.select_id = None;
                        }
                    }
                }
                self.data_map = c.data_map;
                self.select_id = c.select_id;
                for (_, crt) in self.data_map.iter() {
                    self.data_list.push(crt.id.clone());
                }
            }
            None => {}
        }
    }
    pub fn clear(&mut self) {
        self.data_map.clear();
        self.data_list.clear();
        self.persistence.save(
            Path::new("requests").to_path_buf(),
            "data".to_string(),
            &CentralRequestDataListSaved {
                select_id: self.select_id.clone(),
                data_map: self.data_map.clone(),
            },
        );
    }
    pub fn remove(&mut self, id: String) {
        self.data_map.remove(id.as_str());
        self.data_list
            .clone()
            .iter()
            .enumerate()
            .find(|(_, list_id)| list_id.as_str() == id)
            .map(|(index, _)| self.data_list.remove(index));
        self.persistence.save(
            Path::new("requests").to_path_buf(),
            "data".to_string(),
            &CentralRequestDataListSaved {
                select_id: self.select_id.clone(),
                data_map: self.data_map.clone(),
            },
        );
    }
    pub fn add_new(&mut self) {
        let id = Uuid::new_v4().to_string();
        let crt = CentralRequestItem {
            id: id.clone(),
            collection_path: None,
            ..Default::default()
        };
        self.add_crt(crt);
        self.select(id)
    }
    pub fn select(&mut self, id: String) {
        self.select_id = Some(id)
    }
    pub fn add_crt(&mut self, mut crt: CentralRequestItem) {
        crt.set_baseline();
        if !self.data_map.contains_key(crt.id.as_str()) {
            self.data_map.insert(crt.id.clone(), crt.clone());
            self.data_list.push(crt.id.clone())
        }
        self.select(crt.id.clone());
        self.save();
    }
    fn save(&self) {
        self.persistence.save(
            Path::new("requests").to_path_buf(),
            "data".to_string(),
            &CentralRequestDataListSaved {
                select_id: self.select_id.clone(),
                data_map: self.data_map.clone(),
            },
        );
    }
    pub fn auto_save(&self) {
        self.save();
    }

    pub fn update_old_to_new(&mut self, path: String, old_name: String, new_name: String) {
        let old_id = format!("{}/{}", path, old_name);
        let new_id = format!("{}/{}", path, new_name);
        for (index, id) in self.data_list.iter().enumerate() {
            if id == old_id.as_str() {
                self.data_list[index] = new_id.clone();
                break;
            }
        }
        let old_crt = self.data_map.remove(old_id.as_str());
        old_crt.map(|mut crt| {
            crt.rest.name = new_name.clone();
            crt.id = new_id.clone();
            self.data_map.insert(new_id.clone(), crt);
        });
        if self.select_id == Some(old_id.clone()) {
            self.select_id = Some(new_id.clone());
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CentralRequestItem {
    pub id: String,
    pub collection_path: Option<String>,
    pub rest: HttpRecord,
    pub test_result: TestResult,
    pub modify_baseline: String,
}

impl CentralRequestItem {
    pub fn set_baseline(&mut self) {
        self.modify_baseline = self.compute_signature();
    }

    fn compute_signature(&self) -> String {
        format!(
            "{}/{}/{}",
            serde_json::to_string(&self.rest.request).unwrap(),
            self.rest.test_script.clone(),
            self.rest.pre_request_script.clone()
        )
    }

    pub fn is_modify(&self) -> bool {
        let now_sign = self.compute_signature();
        now_sign != self.modify_baseline
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct HistoryDataList {
    persistence: Persistence,
    date_group: BTreeMap<NaiveDate, DateGroupHistoryList>,
}

impl HistoryDataList {
    pub fn load_all(&mut self, workspace: String) -> Result<(), Error> {
        self.persistence.set_workspace(workspace);
        for date_dir in self
            .persistence
            .load_list(Path::new("history").to_path_buf())
            .iter()
        {
            if let Some(date) = date_dir.file_name() {
                if let Some(date_name) = date.to_str() {
                    if let Ok(naive_date) = NaiveDate::from_str(date_name) {
                        let mut date_group_history_list = DateGroupHistoryList::default();
                        for item_path in self.persistence.load_list(date_dir.clone()).iter() {
                            if let Some(history_rest_item) =
                                self.persistence.load(item_path.clone())
                            {
                                date_group_history_list.history_list.push(history_rest_item);
                            }
                        }
                        date_group_history_list
                            .history_list
                            .sort_by(|a, b| b.record_date.cmp(&a.record_date));
                        self.date_group.insert(naive_date, date_group_history_list);
                    }
                }
            }
        }
        Ok(())
    }
    pub fn get_group(&self) -> &BTreeMap<NaiveDate, DateGroupHistoryList> {
        &self.date_group
    }
    pub fn record(&mut self, rest: HttpRecord) {
        let today = Local::now().naive_local().date();
        if !self.date_group.contains_key(&today) {
            self.date_group.insert(
                today,
                DateGroupHistoryList {
                    history_list: vec![],
                },
            );
        }
        let hrt = HistoryRestItem {
            id: Uuid::new_v4().to_string(),
            record_date: Local::now().with_timezone(&Utc),
            rest,
        };
        self.date_group
            .get_mut(&today)
            .unwrap()
            .history_list
            .push(hrt.clone());
        self.persistence.save(
            Path::new("history").join(today.to_string()),
            hrt.id.clone(),
            &hrt,
        );
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct DateGroupHistoryList {
    pub history_list: Vec<HistoryRestItem>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryRestItem {
    pub id: String,
    pub record_date: DateTime<Utc>,
    pub rest: HttpRecord,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpRecord {
    pub name: String,
    pub desc: String,
    pub request: Request,
    #[serde(skip)]
    pub response: Response,
    pub status: ResponseStatus,
    pub pre_request_script: String,
    pub test_script: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Logger {
    pub logs: Vec<Log>,
}

impl Logger {
    pub fn add_info(&mut self, scope: String, msg: String) {
        self.logs.push(Log {
            level: LogLevel::Info,
            time: Local::now(),
            msg,
            scope,
        })
    }
    pub fn add_error(&mut self, scope: String, msg: String) {
        self.logs.push(Log {
            level: LogLevel::Error,
            time: Local::now(),
            msg,
            scope,
        })
    }

    pub fn add_warn(&mut self, scope: String, msg: String) {
        self.logs.push(Log {
            level: LogLevel::Warn,
            time: Local::now(),
            msg,
            scope,
        })
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Log {
    pub level: LogLevel,
    pub time: DateTime<Local>,
    pub msg: String,
    pub scope: String,
}

impl Log {
    pub fn show(&self) -> String {
        format!(
            "{} {:?} [{}] {}",
            self.time.format("%H:%M:%S").to_string(),
            self.level,
            self.scope,
            self.msg
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}
impl HttpRecord {
    pub fn pending(&mut self) {
        self.status = ResponseStatus::Pending;
    }
    pub fn ready(&mut self) {
        self.status = ResponseStatus::Ready;
    }
    pub fn none(&mut self) {
        self.status = ResponseStatus::None;
    }
    pub fn error(&mut self) {
        self.status = ResponseStatus::Error;
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ResponseStatus {
    None,
    Pending,
    Ready,
    Error,
}

impl Default for ResponseStatus {
    fn default() -> Self {
        ResponseStatus::None
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Request {
    pub method: Method,
    pub base_url: String,
    pub params: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub body: HttpBody,
    pub auth: Auth,
}

impl Request {
    pub fn clear_lock_with(&mut self) {
        self.params.retain(|s| s.lock_with == LockWith::NoLock);
        self.headers.retain(|s| s.lock_with == LockWith::NoLock);
        self.body
            .body_form_data
            .retain(|s| s.lock_with == LockWith::NoLock);
        self.body
            .body_xxx_form
            .retain(|s| s.lock_with == LockWith::NoLock);
    }
    pub fn remove_request_content_type(&mut self) {
        self.headers
            .retain(|h| h.key.to_lowercase() != "content-type" || h.lock_with != LockWith::NoLock);
    }
    pub fn set_request_content_type(&mut self, value: String) {
        let mut find = false;
        for header in self.headers.iter_mut() {
            if header.key.to_lowercase() == "content-type" {
                find = true;
                if !header.value.contains(value.as_str()) {
                    header.value = value.clone();
                }
            }
        }
        if !find {
            self.headers.push(Header {
                key: "content-type".to_string(),
                value,
                desc: "".to_string(),
                enable: true,
                lock_with: LockWith::NoLock,
            });
        }
    }
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Auth {
    pub auth_type: AuthType,
    pub basic_username: String,
    pub basic_password: String,
    pub bearer_token: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, EnumIter, EnumString, Display)]
pub enum AuthType {
    InheritAuthFromParent,
    NoAuth,
    BearerToken,
    BasicAuth,
}

impl Auth {
    pub fn get_final_type(&self, auth: Auth) -> AuthType {
        match self.auth_type {
            AuthType::NoAuth => AuthType::NoAuth,
            AuthType::BearerToken => AuthType::BearerToken,
            AuthType::BasicAuth => AuthType::BasicAuth,
            AuthType::InheritAuthFromParent => auth.get_final_type(Auth {
                auth_type: AuthType::NoAuth,
                basic_username: "".to_string(),
                basic_password: "".to_string(),
                bearer_token: "".to_string(),
            }),
        }
    }
    pub fn build_head(
        &self,
        headers: &mut Vec<Header>,
        envs: BTreeMap<String, EnvironmentItemValue>,
        auth: Auth,
    ) {
        let mut header = Header {
            key: "Authorization".to_string(),
            value: "".to_string(),
            desc: "auto gen".to_string(),
            enable: true,
            lock_with: LockWith::LockWithAuto,
        };
        headers.retain(|h| {
            !(h.key.to_lowercase() == "authorization" && h.lock_with != LockWith::NoLock)
        });
        match self.auth_type {
            AuthType::NoAuth => {}
            AuthType::BearerToken => {
                header.value = "Bearer ".to_string()
                    + utils::replace_variable(self.bearer_token.clone(), envs.clone()).as_str();
                headers.push(header)
            }
            AuthType::BasicAuth => {
                let encoded_credentials = general_purpose::STANDARD.encode(format!(
                    "{}:{}",
                    utils::replace_variable(self.basic_username.clone(), envs.clone()),
                    utils::replace_variable(self.basic_password.clone(), envs.clone())
                ));
                header.value = "Basic ".to_string() + encoded_credentials.as_str();
                headers.push(header)
            }
            AuthType::InheritAuthFromParent => auth.build_head(
                headers,
                envs,
                Auth {
                    auth_type: AuthType::NoAuth,
                    basic_username: "".to_string(),
                    basic_password: "".to_string(),
                    bearer_token: "".to_string(),
                },
            ),
        }
    }
}

impl Default for AuthType {
    fn default() -> Self {
        AuthType::InheritAuthFromParent
    }
}

impl HttpRecord {
    pub fn sync(&mut self, envs: BTreeMap<String, EnvironmentItemValue>, auth: Auth) {
        self.request
            .auth
            .build_head(&mut self.request.headers, envs.clone(), auth);
        match self.request.body.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                self.set_request_content_type("multipart/form-data".to_string());
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                self.set_request_content_type("application/x-www-form-urlencoded".to_string());
            }
            BodyType::RAW => match self.request.body.body_raw_type {
                BodyRawType::TEXT => self.set_request_content_type("text/plain".to_string()),
                BodyRawType::JSON => self.set_request_content_type("application/json".to_string()),
                BodyRawType::HTML => self.set_request_content_type("text/html".to_string()),
                BodyRawType::XML => self.set_request_content_type("application/xml".to_string()),
                BodyRawType::JavaScript => {
                    self.set_request_content_type("application/javascript".to_string())
                }
            },
            BodyType::BINARY => {
                let path = Path::new(&self.request.body.body_file);
                let content_type = mime_guess::from_path(path);
                self.set_request_content_type(content_type.first_or_octet_stream().to_string());
            }
        }
    }

    pub fn get_response_content_type(&self) -> Option<Header> {
        self.response
            .headers
            .iter()
            .find(|h| h.key.to_lowercase() == "content-type")
            .cloned()
    }

    pub fn set_request_content_type(&mut self, value: String) {
        self.request.set_request_content_type(value);
    }
    pub fn remove_request_content_type(&mut self) {
        self.request.remove_request_content_type();
    }
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BodyRawType {
    TEXT,
    JSON,
    HTML,
    XML,
    JavaScript,
}

impl Default for BodyRawType {
    fn default() -> Self {
        BodyRawType::JSON
    }
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BodyType {
    NONE,
    FROM_DATA,
    X_WWW_FROM_URLENCODED,
    RAW,
    BINARY,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::NONE
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub lock_with: LockWith,
    pub enable: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LockWith {
    LockWithScript,
    LockWithAuto,
    NoLock,
}

impl Default for LockWith {
    fn default() -> Self {
        LockWith::NoLock
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MultipartData {
    pub data_type: MultipartDataType,
    pub key: String,
    pub value: String,
    pub desc: String,
    pub lock_with: LockWith,
    pub enable: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Display, EnumIter, EnumString, Serialize, Deserialize)]
pub enum MultipartDataType {
    File,
    Text,
}

impl Default for MultipartDataType {
    fn default() -> Self {
        MultipartDataType::Text
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
    pub lock_with: LockWith,
}

impl Header {
    pub fn new_from_tuple(headers: Vec<(String, String)>) -> Vec<Header> {
        let mut result = vec![];
        for (key, value) in headers {
            result.push(Header {
                key,
                value,
                desc: "".to_string(),
                enable: true,
                lock_with: LockWith::NoLock,
            })
        }
        result
    }
    pub fn new_from_map(headers: &HeaderMap) -> Vec<Header> {
        let mut result = vec![];
        for (key, value) in headers.iter() {
            result.push(Header {
                key: key.to_string(),
                value: value.to_str().unwrap().to_string(),
                desc: "".to_string(),
                enable: true,
                lock_with: LockWith::NoLock,
            })
        }
        result
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Response {
    pub body: Arc<HttpBody>,
    pub headers: Vec<Header>,
    pub status: u16,
    pub status_text: String,
    pub elapsed_time: u128,
    #[serde(skip)]
    pub logger: Logger,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpBody {
    pub base64: String,
    pub size: usize,
    pub body_str: String,
    pub body_file: String,
    pub body_type: BodyType,
    pub body_raw_type: BodyRawType,
    pub body_form_data: Vec<MultipartData>,
    pub body_xxx_form: Vec<MultipartData>,
}

impl HttpBody {
    pub fn to_vec(&self) -> Vec<u8> {
        general_purpose::STANDARD.decode(&self.base64).unwrap()
    }
    pub fn get_byte_size(&self) -> String {
        if self.size > 1000000 {
            return (self.size / 1000000).to_string() + " MB";
        } else if self.size > 1000 {
            return (self.size / 1000).to_string() + " KB";
        } else {
            return self.size.to_string() + " B";
        }
    }

    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            base64: general_purpose::STANDARD.encode(&bytes).to_string(),
            size: bytes.len(),
            body_str: "".to_string(),
            body_file: "".to_string(),
            body_type: Default::default(),
            body_raw_type: Default::default(),
            body_form_data: vec![],
            body_xxx_form: vec![],
        }
    }
}
#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: String,
    pub max_age: String,
    pub raw: String,
    pub http_only: bool,
    pub secure: bool,
}

impl Cookie {
    pub fn from_raw(raw: String) -> Self {
        let mut cookie = Cookie {
            name: "".to_string(),
            value: "".to_string(),
            domain: "".to_string(),
            path: "".to_string(),
            expires: "".to_string(),
            max_age: "".to_string(),
            raw,
            http_only: false,
            secure: false,
        };
        let raw = cookie.raw.clone();
        let s = raw.split(";");
        for (index, x) in s.into_iter().enumerate() {
            let one: Vec<&str> = x.splitn(2, "=").collect();
            if one.len() < 2 {
                continue;
            }
            match one[0].trim() {
                "expires" => cookie.expires = one[1].to_string(),
                "path" => cookie.path = one[1].to_string(),
                "domain" => cookie.domain = one[1].to_string(),
                "max-age" => cookie.max_age = one[1].to_string(),
                "secure" => cookie.secure = true,
                "httponly" => cookie.http_only = true,
                _ => {
                    if index == 0 {
                        cookie.value = one[1].to_string();
                        cookie.name = one[0].to_string()
                    }
                }
            }
        }
        cookie
    }
}

#[derive(Debug, Display, PartialEq, EnumString, EnumIter, Clone, Eq, Serialize, Deserialize)]
pub enum Method {
    POST,
    GET,
    PUT,
    PATCH,
    DELETE,
    COPY,
    HEAD,
    OPTIONS,
    LINK,
    UNLINK,
    PURGE,
    LOCK,
    UNLOCK,
    PROPFIND,
    VIEW,
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Export {
    pub export_type: ExportType,
    pub collection: Option<Collection>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ExportType {
    Collection,
    Request,
    Environment,
}

impl Default for ExportType {
    fn default() -> Self {
        ExportType::Collection
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub status: TestStatus,
    open_test: Option<String>,
    append: Vec<TestAssertResult>,
    pub test_info_list: Vec<TestInfo>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestInfo {
    pub name: String,
    pub results: Vec<TestAssertResult>,
    pub status: TestStatus,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestAssertResult {
    pub assert_result: TestStatus,
    pub msg: String,
}

impl TestResult {
    pub fn open(&mut self, name: String) {
        self.open_test = Some(name);
    }
    pub fn close(&mut self, name: String) {
        self.open_test = None;
        let mut status = TestStatus::PASS;
        for tar in self.append.iter() {
            if tar.assert_result != TestStatus::PASS {
                status = TestStatus::FAIL;
                break;
            }
        }
        self.test_info_list.push(TestInfo {
            name,
            results: self.append.clone(),
            status,
        });
        self.append.clear();
        self.status = TestStatus::PASS;
        for ti in self.test_info_list.iter() {
            if ti.status == TestStatus::FAIL {
                self.status = TestStatus::FAIL;
                break;
            }
        }
    }
    pub fn append(&mut self, assert_result: bool, msg: String) {
        if assert_result {
            self.append.push(TestAssertResult {
                assert_result: TestStatus::PASS,
                msg,
            });
        } else {
            self.append.push(TestAssertResult {
                assert_result: TestStatus::FAIL,
                msg,
            });
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Display)]
pub enum TestStatus {
    None,
    PASS,
    FAIL,
}

impl Default for TestStatus {
    fn default() -> Self {
        TestStatus::None
    }
}
