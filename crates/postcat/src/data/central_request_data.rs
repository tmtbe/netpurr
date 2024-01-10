use std::path::Path;

use eframe::epaint::ahash::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::http::HttpRecord;
use crate::data::test::TestResult;
use crate::persistence::{Persistence, PersistenceItem};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestDataList {
    pub select_id: Option<String>,
    pub data_list: Vec<String>,
    pub data_map: HashMap<String, CentralRequestItem>,
    persistence: Persistence,
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
    pub fn update_old_id_to_new(&mut self, old_id: String, path: String, new_name: String) {
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
    pub fn update_old_name_to_new_name(
        &mut self,
        path: String,
        old_name: String,
        new_name: String,
    ) {
        let old_id = format!("{}/{}", path, old_name);
        self.update_old_id_to_new(old_id, path, new_name)
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
    pub fn get_tab_name(&self) -> String {
        if self.rest.name != "" {
            self.rest.name.clone()
        } else {
            if self
                .rest
                .request
                .base_url
                .trim()
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                == ""
            {
                "Untitled Request".to_string()
            } else {
                self.rest.request.base_url.clone()
            }
        }
    }
    pub fn set_baseline(&mut self) {
        self.modify_baseline = self.compute_signature();
    }

    fn compute_signature(&self) -> String {
        format!(
            "Request:[{}] TestScript:[{}] PreRequestScript:[{}]",
            &self.rest.request.compute_signature(),
            self.rest.test_script.clone(),
            self.rest.pre_request_script.clone()
        )
    }

    pub fn is_modify(&self) -> bool {
        let now_sign = self.compute_signature();
        now_sign != self.modify_baseline
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(default)]
struct CentralRequestDataListSaved {
    pub select_id: Option<String>,
    pub data_map: HashMap<String, CentralRequestItem>,
}
