use crate::models::history_model::HistoryRestData;

pub enum MailEvent {
    String(String),
    AddHttpRecord(HistoryRestData)
}
