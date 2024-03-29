use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::environment::EnvironmentItemValue;
use crate::data::http::{Header, LockWith};
use crate::utils;

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
    pub fn compute_signature(&self) -> String {
        format!(
            "Type:{} BasicUsername:{} BasicPassword:{} BearerToken:{}",
            self.auth_type, self.basic_username, self.basic_password, self.bearer_token
        )
    }
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
