use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct HtmlReport {
    #[serde(rename = "testPass")]
    pub test_pass: usize,

    #[serde(rename = "testSkip")]
    pub test_skip: usize,

    #[serde(rename = "totalTime")]
    pub total_time: String,

    #[serde(rename = "testAll")]
    pub test_all: usize,

    #[serde(rename = "beginTime")]
    pub begin_time: String,

    #[serde(rename = "testResult")]
    pub test_result: Vec<HtmlReportTestResult>,

    #[serde(rename = "testFail")]
    pub test_fail: usize,

    #[serde(rename = "testName")]
    pub test_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct HtmlReportTestResult {
    #[serde(rename = "log")]
    pub log: Vec<String>,

    #[serde(rename = "methodName")]
    pub method_name: String,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "className")]
    pub class_name: String,

    #[serde(rename = "spendTime")]
    pub spend_time: String,

    #[serde(rename = "status")]
    pub status: String,
}
