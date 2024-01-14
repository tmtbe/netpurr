use deno_core::anyhow::Error;
use egui::Ui;
use poll_promise::Promise;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::script::script::{Context, ScriptScope};
use crate::utils;

#[derive(Default)]
pub struct TestScriptWindows {
    test_windows_open: bool,
    script_scopes: Vec<ScriptScope>,
    context: Option<Context>,
    run_result: Option<Promise<Result<Context, Error>>>,
}

impl Window for TestScriptWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("TEST SCRIPT")
            .modal(true)
            .max_width(500.0)
            .min_height(400.0)
            .max_height(400.0)
            .collapsible(false)
            .resizable(true)
    }

    fn set_open(&mut self, open: bool) {
        self.test_windows_open = open;
    }

    fn get_open(&self) -> bool {
        self.test_windows_open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        _: &mut ConfigData,
        _: &mut WorkspaceData,
        operation: Operation,
    ) {
        ui.vertical(|ui| match &self.run_result {
            None => {
                if ui.button("Run Script").clicked() {
                    if let Some(context) = &self.context {
                        self.run_result = Some(
                            operation
                                .script_runtime()
                                .run(self.script_scopes.clone(), context.clone()),
                        );
                    }
                }
                ui.separator();
            }
            Some(result) => {
                ui.add_enabled_ui(false, |ui| {
                    ui.button("Run Script");
                    ui.separator();
                });
                match result.ready() {
                    None => {
                        ui.ctx().request_repaint();
                    }
                    Some(js_run_result) => match js_run_result {
                        Ok(new_context) => {
                            Self::render_logs(ui, new_context);
                        }
                        Err(e) => {
                            ui.strong("Run Error:");
                            let mut msg = e.to_string();
                            utils::text_edit_multiline_justify(ui, &mut msg);
                        }
                    },
                }
            }
        });
    }
}
impl TestScriptWindows {
    pub fn with(mut self, script_scopes: Vec<ScriptScope>, context: Context) -> Self {
        self.test_windows_open = true;
        self.script_scopes = script_scopes;
        self.context = Some(context);
        self.run_result = None;
        self
    }
}

impl TestScriptWindows {
    fn render_logs(ui: &mut Ui, new_context: &Context) {
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "log");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        if new_context.logger.logs.len() > 0 {
            ui.strong("Output Log:");
            ui.push_id("log_info", |ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for log in new_context.logger.logs.iter() {
                            let mut content = format!("> {}", log.show());
                            egui::TextEdit::multiline(&mut content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_rows(1)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                                .layouter(&mut layouter)
                                .show(ui);
                        }
                    });
            });
        }
    }
}
