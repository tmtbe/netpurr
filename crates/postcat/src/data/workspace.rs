use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use reqwest::blocking::Client;
use rustygit::Repository;

use crate::data::auth::{Auth, AuthType};
use crate::data::central_request_data::{CentralRequestDataList, CentralRequestItem};
use crate::data::collections::{Collection, Collections};
use crate::data::cookies_manager::CookiesManager;
use crate::data::environment::{Environment, EnvironmentItemValue};
use crate::data::history::HistoryDataList;
use crate::data::http::HttpRecord;
use crate::script::script::ScriptScope;

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
    pub fn build_http_client(&mut self) -> Client {
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

    pub fn get_collection(&self, option_path: Option<String>) -> Option<Collection> {
        let path = option_path?;
        let collection_name = path.splitn(2, "/").next()?;
        self.collections.data.get(collection_name).cloned()
    }
    pub fn save_crt(
        &mut self,
        crt_id: String,
        collection_path: String,
        modify_http_record: impl FnOnce(&mut HttpRecord),
    ) {
        let mut new_name_option = None;
        self.central_request_data_list
            .data_map
            .get_mut(crt_id.as_str())
            .map(|crt| {
                let (_, cf_option) = self
                    .collections
                    .get_mut_folder_with_path(collection_path.clone());
                cf_option.map(|cf| {
                    let mut http_record = crt.rest.clone();
                    modify_http_record(&mut http_record);
                    new_name_option = Some(http_record.name.clone());
                    self.collections.insert_http_record(cf.clone(), http_record);
                    crt.set_baseline();
                });
            });
        new_name_option.map(|new_name| {
            self.central_request_data_list.update_old_id_to_new(
                crt_id,
                collection_path.clone(),
                new_name.clone(),
            );
        });
    }
    pub fn get_crt(&self, id: String) -> &CentralRequestItem {
        self.central_request_data_list
            .data_map
            .get(id.as_str())
            .unwrap()
    }
    pub fn get_mut_crt(&mut self, id: String) -> &mut CentralRequestItem {
        self.central_request_data_list
            .data_map
            .get_mut(id.as_str())
            .unwrap()
    }
    pub fn get_crt_envs(&self, id: String) -> BTreeMap<String, EnvironmentItemValue> {
        let crt = self.get_crt(id);
        self.environment
            .get_variable_hash_map(self.get_collection(crt.collection_path.clone()))
    }

    pub fn get_crt_parent_auth(&self, id: String) -> Auth {
        let crt = self.get_crt(id);
        match &crt.collection_path {
            None => Auth {
                auth_type: AuthType::NoAuth,
                basic_username: "".to_string(),
                basic_password: "".to_string(),
                bearer_token: "".to_string(),
            },
            Some(collection_path) => self.collections.get_auth(collection_path.clone()),
        }
    }

    pub fn get_crt_parent_scripts(&self, id: String) -> (Option<ScriptScope>, Option<ScriptScope>) {
        let crt = self.get_crt(id);
        let mut pre_request_script_scope = None;
        let mut test_script_scope = None;
        match &crt.collection_path {
            None => {}
            Some(collection_path) => {
                pre_request_script_scope = self
                    .collections
                    .get_pre_request_script_scope(collection_path.clone());
                test_script_scope = self
                    .collections
                    .get_test_script_scope(collection_path.clone());
            }
        }
        (pre_request_script_scope, test_script_scope)
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
}
