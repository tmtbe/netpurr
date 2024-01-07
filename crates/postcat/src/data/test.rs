use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub status: TestStatus,
    open_test: Option<String>,
    append: Vec<TestAssertResult>,
    pub test_info_list: Vec<TestInfo>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestInfo {
    pub name: String,
    pub results: Vec<TestAssertResult>,
    pub status: TestStatus,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestAssertResult {
    pub assert_result: TestStatus,
    pub msg: String,
}

impl TestResult {
    pub fn open(&mut self, name: String) {
        self.open_test = Some(name);
    }
    pub fn close(&mut self, name: String) {
        self.open_test = None;
        let mut status = TestStatus::PASS;
        for tar in self.append.iter() {
            if tar.assert_result != TestStatus::PASS {
                status = TestStatus::FAIL;
                break;
            }
        }
        self.test_info_list.push(TestInfo {
            name,
            results: self.append.clone(),
            status,
        });
        self.append.clear();
        self.status = TestStatus::PASS;
        for ti in self.test_info_list.iter() {
            if ti.status == TestStatus::FAIL {
                self.status = TestStatus::FAIL;
                break;
            }
        }
    }
    pub fn append(&mut self, assert_result: bool, msg: String) {
        if assert_result {
            self.append.push(TestAssertResult {
                assert_result: TestStatus::PASS,
                msg,
            });
        } else {
            self.append.push(TestAssertResult {
                assert_result: TestStatus::FAIL,
                msg,
            });
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Display)]
pub enum TestStatus {
    None,
    PASS,
    FAIL,
}

impl Default for TestStatus {
    fn default() -> Self {
        TestStatus::None
    }
}
