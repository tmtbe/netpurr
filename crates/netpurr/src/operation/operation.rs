use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use egui::{emath, WidgetText};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use poll_promise::Promise;

use netpurr_core::data::collections::{CollectionFolder, Testcase};
use netpurr_core::data::cookies_manager::CookiesManager;
use netpurr_core::data::environment::EnvironmentItemValue;
use netpurr_core::data::http::Request;
use netpurr_core::data::record::Record;
use netpurr_core::data::websocket::WebSocketSession;
use netpurr_core::runner::{
    RunRequestInfo, Runner, TestGroupRunResults, TestRunError, TestRunResult,
};
use netpurr_core::script::{Context, ScriptScope, ScriptTree};

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::git::Git;
use crate::operation::windows::{Window, Windows};

#[derive(Clone)]
pub struct Operation {
    runner: Runner,
    lock_ui: HashMap<String, bool>,
    modal_flag: Rc<RefCell<ModalFlag>>,
    toasts: Rc<RefCell<Toasts>>,
    current_windows: Rc<RefCell<Windows>>,
    add_windows: Rc<RefCell<Windows>>,
    git: Git,
}

#[derive(Default)]
pub struct ModalFlag {
    lock_ui: HashMap<String, bool>,
}

impl ModalFlag {
    pub fn lock_ui(&mut self, key: String, bool: bool) {
        self.lock_ui.insert(key, bool);
    }
    pub fn get_ui_lock(&self) -> bool {
        let mut result = false;
        for (_, lock) in self.lock_ui.iter() {
            result = result || (lock.clone());
        }
        result
    }
}

impl Operation {
    pub fn new(cookies_manager: CookiesManager) -> Self {
        Operation {
            lock_ui: Default::default(),
            runner: Runner::new(cookies_manager.cookie_store.clone()),
            modal_flag: Rc::new(RefCell::new(ModalFlag::default())),
            toasts: Rc::new(RefCell::new(
                Toasts::default()
                    .anchor(emath::Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                    .direction(egui::Direction::BottomUp),
            )),
            current_windows: Rc::new(RefCell::new(Windows::default())),
            add_windows: Rc::new(RefCell::new(Default::default())),
            git: Default::default(),
        }
    }
    pub fn send_rest_with_script_promise(
        &self,
        run_request_info: RunRequestInfo,
    ) -> Promise<Result<TestRunResult, TestRunError>> {
        self.runner.send_rest_with_script_promise(run_request_info)
    }

    pub fn run_test_group_promise(
        &self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        script_tree: ScriptTree,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_path: String,
        parent_testcase: Option<Testcase>,
        folder: Rc<RefCell<CollectionFolder>>,
    ) -> Promise<()> {
        self.runner.run_test_group_promise(
            envs,
            script_tree,
            test_group_run_result,
            collection_path,
            parent_testcase,
            folder,
        )
    }

    pub fn run_test_record_promise(
        &self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        script_tree: ScriptTree,
        test_group_run_result: Arc<RwLock<TestGroupRunResults>>,
        collection_path: String,
        parent_testcase: Option<Testcase>,
        record: Record,
    ) -> Promise<()> {
        self.runner.run_test_record_promise(
            envs,
            script_tree,
            test_group_run_result,
            collection_path,
            parent_testcase,
            record,
        )
    }

    pub fn connect_websocket_with_script(
        &self,
        http_request: Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
        pre_request_scripts: Vec<ScriptScope>,
        test_scripts: Vec<ScriptScope>,
    ) -> WebSocketSession {
        self.runner.connect_websocket_with_script(RunRequestInfo {
            collection_path: None,
            request_name: "".to_string(),
            request: http_request,
            envs,
            pre_request_scripts,
            test_scripts,
            testcase: Default::default(),
        })
    }

    pub fn run_script(
        &self,
        scripts: Vec<ScriptScope>,
        context: Context,
    ) -> Promise<anyhow::Result<Context>> {
        self.runner.run_script(scripts, context)
    }

    pub fn lock_ui(&self, key: String, bool: bool) {
        self.modal_flag.borrow_mut().lock_ui(key, bool);
    }
    pub fn get_ui_lock(&self) -> bool {
        self.modal_flag.borrow_mut().get_ui_lock()
    }

    pub fn add_toast(&self, toast: Toast) {
        self.toasts.borrow_mut().add(toast);
    }

    pub fn add_success_toast(&self, text: impl Into<WidgetText>) {
        self.add_toast(Toast {
            text: text.into(),
            kind: ToastKind::Success,
            options: ToastOptions::default()
                .show_icon(true)
                .duration_in_seconds(2.0)
                .show_progress(true),
        });
    }
    pub fn add_error_toast(&self, text: impl Into<WidgetText>) {
        self.add_toast(Toast {
            text: text.into(),
            kind: ToastKind::Error,
            options: ToastOptions::default()
                .show_icon(true)
                .duration_in_seconds(5.0)
                .show_progress(true),
        });
    }
    pub fn add_window(&self, window: Box<dyn Window>) {
        self.add_windows
            .borrow_mut()
            .add(Rc::new(RefCell::new(window)));
    }

    pub fn show(
        &self,
        ctx: &egui::Context,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
    ) {
        self.toasts.borrow_mut().show(ctx);
        for w in &self.add_windows.borrow().show_windows {
            self.current_windows.borrow_mut().add(w.clone())
        }
        self.add_windows.borrow_mut().show_windows.clear();
        self.current_windows
            .borrow()
            .show(ctx, config_data, workspace_data, self.clone());
        self.current_windows.borrow_mut().retain()
    }
    pub fn git(&self) -> &Git {
        &self.git
    }
}
