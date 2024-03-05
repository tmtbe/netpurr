use std::collections::BTreeMap;
use std::str::FromStr;

use regex::Regex;

use crate::data::environment::{EnvironmentItemValue, EnvironmentValueType};
use crate::data::environment_function::{get_env_result, EnvFunction};

pub fn replace_variable(content: String, envs: BTreeMap<String, EnvironmentItemValue>) -> String {
    let re = Regex::new(r"\{\{.*?}}").unwrap();
    let mut result = content.clone();
    loop {
        let temp = result.clone();
        let find = re.find_iter(temp.as_str()).next();
        match find {
            None => break,
            Some(find_match) => {
                let key = find_match
                    .as_str()
                    .trim_start_matches("{{")
                    .trim_end_matches("}}")
                    .trim_start()
                    .trim_end();
                let v = envs.get(key);
                match v {
                    None => result.replace_range(find_match.range(), "{UNKNOWN}"),
                    Some(etv) => match etv.value_type {
                        EnvironmentValueType::String => {
                            result.replace_range(find_match.range(), etv.value.as_str())
                        }
                        EnvironmentValueType::Function => {
                            let env_func = EnvFunction::from_str(etv.value.as_str());
                            match env_func {
                                Ok(f) => result
                                    .replace_range(find_match.range(), get_env_result(f).as_str()),
                                Err(_) => {
                                    result.replace_range(find_match.range(), "{UNKNOWN}");
                                }
                            }
                        }
                    },
                }
            }
        }
    }
    result
}
