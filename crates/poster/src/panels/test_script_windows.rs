use deno_core::anyhow::Error;
use egui::Ui;
use poll_promise::Promise;

use crate::operation::Operation;
use crate::script::script::Context;
use crate::utils;

#[derive(Default)]
pub struct TestScriptWindows {
    test_windows_open: bool,
    script: String,
    context: Option<Context>,
    run_result: Option<Promise<Result<Context, Error>>>,
}

impl TestScriptWindows {
    pub(crate) fn open(&mut self, script: String, context: Context) {
        self.test_windows_open = true;
        self.script = script;
        self.context = Some(context);
        self.run_result = None;
    }
}

impl TestScriptWindows {
    pub fn set_and_render(&mut self, ui: &mut Ui, operation: &mut Operation) {
        operation.lock_ui("test_script".to_string(), self.test_windows_open);
        let mut test_windows_open = self.test_windows_open;
        egui::Window::new("TEST SCRIPT")
            .default_open(true)
            .max_width(500.0)
            .min_height(400.0)
            .max_height(400.0)
            .collapsible(false)
            .resizable(true)
            .open(&mut test_windows_open)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| match &self.run_result {
                    None => {
                        if ui.button("Run Script").clicked() {
                            if let Some(context) = &self.context {
                                self.run_result = Some(
                                    operation
                                        .script_runtime()
                                        .run(self.script.clone(), context.clone()),
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
                })
            });
        self.test_windows_open = test_windows_open;
    }

    fn render_logs(ui: &mut Ui, new_context: &Context) {
        if new_context.logger.infos().len() > 0 {
            ui.strong("Output Log:");
            ui.push_id("log_info", |ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for info in new_context.logger.infos() {
                            ui.label("> ".to_string() + info.as_str());
                        }
                    });
            });
        }
        if new_context.logger.errors().len() > 0 {
            ui.strong("Output Error:");
            ui.push_id("log_error", |ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for error in new_context.logger.errors() {
                            ui.label("> ".to_string() + error.as_str());
                        }
                    });
            });
        }
    }
}
