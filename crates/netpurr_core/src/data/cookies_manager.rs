use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::sync::Arc;

use log::error;
use serde::{Deserialize, Serialize};
use url::Url;

use cookie_store::CookieDomain;
use reqwest_cookie_store::{CookieStoreMutex, RawCookie};

use crate::persistence::{Persistence, PersistenceItem};

#[derive(Default, Clone, Debug)]
pub struct CookiesManager {
    persistence: Persistence,
    pub cookie_store: Arc<CookieStoreMutex>,
}

impl CookiesManager {
    pub fn load_all(&mut self, workspace: String) {
        self.persistence.set_workspace(workspace);
        let cookie_store = {
            if let Ok(file) = File::open(self.persistence.get_workspace_dir().join("cookies.json"))
                .map(std::io::BufReader::new)
            {
                // use re-exported version of `CookieStore` for crate compatibility
                reqwest_cookie_store::CookieStore::load_json(file)
                    .unwrap_or(reqwest_cookie_store::CookieStore::default())
            } else {
                reqwest_cookie_store::CookieStore::default()
            }
        };
        let cookie_store = CookieStoreMutex::new(cookie_store);
        self.cookie_store = Arc::new(cookie_store);
    }

    pub fn save(&self) {
        match self.cookie_store.lock() {
            Ok(store) => {
                let mut writer =
                    File::create(self.persistence.get_workspace_dir().join("cookies.json"))
                        .map(std::io::BufWriter::new)
                        .unwrap();
                match store.save_incl_expired_and_nonpersistent_json(&mut writer) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("{}", err)
                    }
                }
            }
            Err(err) => {
                error!("{}", err)
            }
        }
    }
    pub fn contain_domain(&self, domain: String) -> bool {
        match self.cookie_store.lock() {
            Ok(store) => store
                .iter_any()
                .find(|c| {
                    c.domain == CookieDomain::HostOnly(domain.clone())
                        || c.domain == CookieDomain::Suffix(domain.clone())
                })
                .is_some(),
            Err(_) => false,
        }
    }
    pub fn contain_domain_key(&self, domain: String, key: String) -> bool {
        match self.cookie_store.lock() {
            Ok(store) => store
                .iter_any()
                .find(|c| c.domain() == Some(domain.as_str()) && c.name() == key.as_str())
                .is_some(),
            Err(_) => false,
        }
    }
    pub fn get_url_cookies(&self, mut url: String) -> BTreeMap<String, Cookie> {
        let mut result = BTreeMap::new();
        let url_parse = Url::parse(url.as_str());
        match url_parse {
            Ok(url_parse_ok) => match self.cookie_store.lock() {
                Ok(store) => {
                    store
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
                }
                Err(_) => {}
            },
            Err(_) => {}
        }
        result
    }
    pub fn get_domain_cookies(&self, domain: String) -> Option<BTreeMap<String, Cookie>> {
        let mut result = BTreeMap::new();
        match self.cookie_store.lock() {
            Ok(store) => {
                store
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
            }
            Err(_) => {}
        }
        Some(result)
    }
    pub fn add_domain_cookies(&self, cookie: Cookie) -> Result<(), String> {
        let result = match &RawCookie::parse(cookie.raw.as_str()) {
            Ok(c) => match self.cookie_store.lock() {
                Ok(mut store) => match store.insert_raw_no_url_check(c) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                },
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
                match self.cookie_store.lock() {
                    Ok(mut store) => match store.insert_raw_no_url_check(c) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    },
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
        match self.cookie_store.lock() {
            Ok(mut store) => {
                store.remove_domain(domain.as_str());
                self.save()
            }
            Err(_) => {}
        }
    }
    pub fn remove_domain_path_name(&mut self, mut domain: String, path: String, name: String) {
        match self.cookie_store.lock() {
            Ok(mut store) => {
                store.remove(domain.as_str(), path.as_str(), name.as_str());
                self.save()
            }
            Err(_) => {}
        }
    }
    pub fn get_cookie_domains(&self) -> Vec<String> {
        let mut hash_set: BTreeSet<String> = BTreeSet::new();
        match self.cookie_store.lock() {
            Ok(store) => {
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
            }
            Err(_) => {}
        }
        let mut keys = vec![];
        for domain_name in hash_set {
            keys.push(domain_name)
        }
        keys
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
