use std::cell::RefCell;
use std::ops::Deref;
use std::process::exit;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use clap::Parser;
use futures_util::future::join_all;
use reqwest::Client;

use netpurr_core::data::collections::{CollectionFolder, CollectionFolderOnlyRead, Testcase};
use netpurr_core::data::test::TestStatus;
use netpurr_core::data::workspace_data::WorkspaceData;
use netpurr_core::runner;
use netpurr_core::runner::test::ResultTreeFolder;
use netpurr_core::runner::TestGroupRunResults;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    workspace_name: String,
    #[arg(short, long)]
    collection_name: String,
}

fn main() {
    //let args = Args::parse();
    let args=Args{
        workspace_name: "aiproject".to_string(),
        collection_name: "百炼".to_string(),
    };
    let client = Client::builder()
        .trust_dns(true)
        .tcp_nodelay(true)
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap_or_default();
    let mut workspace_data = WorkspaceData::default();
    workspace_data.load_all(args.workspace_name.clone());
    let test_group_run_results = Arc::new(RwLock::new(TestGroupRunResults::default()));
    let collection_op = workspace_data.get_collection_by_name(args.collection_name.clone());
    match collection_op {
        None => {
            println!("{}", "collection is not exist")
        }
        Some(collection) => run_test_group(
            client,
            workspace_data,
            test_group_run_results,
            args.collection_name.clone(),
            collection.folder.borrow().get_path(),
            None,
            collection.folder.clone(),
        ),
    }
}
fn run_test_group(
    client: Client,
    workspace_data: WorkspaceData,
    test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
    collection_name: String,
    collection_path: String,
    parent_testcase: Option<Testcase>,
    folder: Rc<RefCell<CollectionFolder>>,
) {
    let envs =
        workspace_data.get_build_envs(workspace_data.get_collection(Some(collection_name.clone())));
    let script_tree = workspace_data.get_script_tree(collection_path.clone());
    let folder_only_read = CollectionFolderOnlyRead::from(folder.clone());
    let run_request_infos = runner::Runner::get_test_group_jobs(
        envs.clone(),
        script_tree.clone(),
        collection_path.clone(),
        parent_testcase,
        folder_only_read.clone(),
    );
    runner::Runner::run_test_group_jobs(client,run_request_infos,test_group_run_result.clone(),true);
    let result_tree = ResultTreeFolder::create(
        folder.clone(),
        vec![],
        test_group_run_result.read().unwrap().deref().clone(),
    );
    let json = serde_yaml::to_string(&result_tree).expect("yaml error");
    println!("{}", json);
    if result_tree.status == TestStatus::PASS {
        println!("{}", "Test Success");
        exit(0);
    } else {
        println!("{}", "Test Error");
        exit(1);
    }
}
