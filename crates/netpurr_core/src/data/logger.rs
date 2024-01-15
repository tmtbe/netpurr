use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

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
