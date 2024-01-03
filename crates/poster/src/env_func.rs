use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

#[derive(EnumIter, EnumString, Display)]
pub enum EnvFunction {
    RandomInt,
    UUID,
    Timestamp,
}

pub fn get_env_result(name: EnvFunction) -> String {
    match name {
        EnvFunction::RandomInt => {
            let i: i32 = rand::thread_rng().gen_range(0..i32::MAX);
            i.to_string()
        }
        EnvFunction::UUID => Uuid::new_v4().to_string(),
        EnvFunction::Timestamp => {
            let current_time = SystemTime::now();
            let timestamp = current_time
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let timestamp_seconds = timestamp.as_secs();
            timestamp_seconds.to_string()
        }
    }
}
