use crate::models::http::HttpRecord;

#[derive(Debug)]
pub enum MailEvent {
    String(String),
    HttpRecord(HttpRecord)
}
