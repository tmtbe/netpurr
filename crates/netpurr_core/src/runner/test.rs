use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::data::collections::{CollectionFolder, Testcase};
use crate::data::test::TestStatus;
use crate::runner::{TestGroupRunResults, TestRunError, TestRunResult};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ResultTreeFolder {
    pub status: TestStatus,
    pub name: String,
    pub cases: BTreeMap<String, ResultTreeCase>,
}
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ResultTreeCase {
    pub status: TestStatus,
    pub name: String,
    pub folders: BTreeMap<String, ResultTreeFolder>,
    pub requests: Vec<ResultTreeRequest>,
}
impl ResultTreeCase {
    pub fn get_success_count(&self) -> i32 {
        let mut success_count = 0;
        for r in self.requests.iter() {
            if r.status == TestStatus::PASS {
                success_count = success_count + 1;
            }
        }
        for (_, f) in self.folders.iter() {
            success_count += f.get_success_count();
        }
        success_count
    }
    pub fn get_total_count(&self) -> i32 {
        let mut success_count = 0;

        for r in self.requests.iter() {
            success_count = success_count + 1;
        }
        for (_, f) in self.folders.iter() {
            success_count += f.get_total_count();
        }

        success_count
    }
}
impl ResultTreeFolder {
    pub fn create(
        folder: Rc<RefCell<CollectionFolder>>,
        testcase_paths: Vec<String>,
        results: TestGroupRunResults,
    ) -> Self {
        let mut folder_status = TestStatus::WAIT;
        let mut testcases = folder.borrow().testcases.clone();
        let folder_name = folder.borrow().name.clone();
        if testcases.is_empty() {
            let testcase = Testcase::default();
            testcases.insert(testcase.name.clone(), testcase);
        }
        let mut new_result_tree_folder = ResultTreeFolder {
            status: folder_status.clone(),
            name: folder.borrow().name.clone(),
            cases: Default::default(),
        };
        folder_status = TestStatus::PASS;
        for (folder_testcase_name, folder_testcase) in testcases.iter() {
            let mut case_status = TestStatus::WAIT;
            let mut case_folders = BTreeMap::new();
            let mut case_requests = vec![];
            case_status = TestStatus::PASS;
            let mut new_folder_testcase_nodes = testcase_paths.clone();
            new_folder_testcase_nodes.push(format!("{}:{}", folder_name, folder_testcase_name));
            for (name, f) in folder.borrow().folders.iter() {
                let child_folder = ResultTreeFolder::create(
                    f.clone(),
                    new_folder_testcase_nodes.clone(),
                    results.clone(),
                );
                match &child_folder.status {
                    TestStatus::None => {}
                    TestStatus::WAIT => case_status = TestStatus::WAIT,
                    TestStatus::PASS => {}
                    TestStatus::FAIL => case_status = TestStatus::FAIL,
                    TestStatus::SKIP => case_status = TestStatus::SKIP,
                    TestStatus::RUNNING => case_status = TestStatus::WAIT,
                }
                case_folders.insert(name.to_string(), child_folder);
            }
            for (request_name, record) in folder.borrow().requests.iter() {
                let mut record_testcases = record.testcase().clone();
                if record_testcases.is_empty() {
                    let testcase = Testcase::default();
                    record_testcases.insert(testcase.name.clone(), testcase);
                }
                for (request_testcase_name, _) in record_testcases.iter() {
                    let mut request_testcase_path = new_folder_testcase_nodes.clone();
                    request_testcase_path
                        .push(format!("{}:{}", request_name, request_testcase_name));
                    let result = results.find(request_testcase_path);
                    let mut request_status = TestStatus::WAIT;
                    match &result {
                        None => {
                            case_status = TestStatus::WAIT;
                        }
                        Some(rr) => match rr {
                            Ok(r) => {
                                request_status = r.test_result.status.clone();
                                if request_status == TestStatus::FAIL {
                                    case_status = TestStatus::FAIL;
                                }else if request_status == TestStatus::RUNNING {
                                    case_status = TestStatus::RUNNING;
                                }
                            }
                            Err(e) => {
                                request_status = TestStatus::FAIL;
                                case_status = TestStatus::FAIL;
                            }
                        },
                    }
                    case_requests.push(ResultTreeRequest {
                        name: format!("{}:{}", request_name, request_testcase_name),
                        status: request_status,
                        result: result.clone(),
                    });
                }
            }
            let result_tree_case = ResultTreeCase {
                status: case_status.clone(),
                name: folder_testcase.name.to_string(),
                folders: case_folders.clone(),
                requests: case_requests.clone(),
            };
            new_result_tree_folder
                .cases
                .insert(folder_testcase.name.to_string(), result_tree_case);
            match &case_status {
                TestStatus::None => {}
                TestStatus::WAIT => folder_status = TestStatus::WAIT,
                TestStatus::PASS => {}
                TestStatus::FAIL => folder_status = TestStatus::FAIL,
                TestStatus::SKIP => folder_status = TestStatus::SKIP,
                TestStatus::RUNNING => folder_status = TestStatus::WAIT
            }
        }
        new_result_tree_folder.status = folder_status.clone();
        new_result_tree_folder
    }

    pub fn get_success_count(&self) -> i32 {
        let mut success_count = 0;
        for (_, case) in self.cases.iter() {
            for r in case.requests.iter() {
                if r.status == TestStatus::PASS {
                    success_count = success_count + 1;
                }
            }
            for (_, f) in case.folders.iter() {
                success_count += f.get_success_count();
            }
        }
        success_count
    }
    pub fn get_total_count(&self) -> i32 {
        let mut success_count = 0;
        for (_, case) in self.cases.iter() {
            for r in case.requests.iter() {
                success_count = success_count + 1;
            }
            for (_, f) in case.folders.iter() {
                success_count += f.get_total_count();
            }
        }
        success_count
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ResultTreeRequest {
    pub name: String,
    pub status: TestStatus,
    pub result: Option<Result<TestRunResult, TestRunError>>,
}
