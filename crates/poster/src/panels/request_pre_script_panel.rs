use std::ops::Add;

use egui::Ui;

use crate::data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::test_script_windows::TestScriptWindows;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::script::script::{Context, Logger};

#[derive(Default)]
pub struct RequestPreScriptPanel {
    test_script_windows: TestScriptWindows,
}

impl DataView for RequestPreScriptPanel {
    type CursorType = String;

    fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    ) {
        let (data, envs, auth) = workspace_data.get_mut_crt_and_envs_auth(cursor.clone());
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "js");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        ui.horizontal(|ui| {
            egui::SidePanel::right("pre_request_right")
                .resizable(true)
                .min_width(300.0)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.label("Pre-request scripts are written in JavaScript， and are run before the request is sent.");
                    if ui.link("Test").clicked() {
                        let js = data.rest.pre_request_script.clone();
                        let context = Context {
                            request: data.rest.request.clone(),
                            envs,
                            logger: Logger::default(),
                        };
                        self.test_script_windows.open(js, context);
                    }
                    ui.separator();
                    ui.strong("SNIPPETS");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if ui.link("Log info message").clicked() {
                            data.rest.pre_request_script = data.rest.pre_request_script.clone().add("\nconsole.log(\"info1\",\"info2\");");
                        }
                        if ui.link("Log error message").clicked() {
                            data.rest.pre_request_script = data.rest.pre_request_script.clone().add("\nconsole.error(\"error1\",\"error2\");");
                        }
                        if ui.link("Get a variable").clicked() {
                            data.rest.pre_request_script = data.rest.pre_request_script.clone().add("\nposter.get_env(\"variable_key\");");
                        }
                        if ui.link("Set a variable").clicked() {
                            data.rest.pre_request_script = data.rest.pre_request_script.clone().add("\nposter.set_env(\"variable_key\",\"variable_value\");");
                        }
                        if ui.link("Add a header").clicked() {
                            data.rest.pre_request_script = data.rest.pre_request_script.clone().add("\nposter.add_header(\"header_key\",\"header_value\");");
                        }
                        if ui.link("Add a params").clicked() {
                            data.rest.pre_request_script = data.rest.pre_request_script.clone().add("\nposter.add_params(\"params_key\",\"params_value\");");
                        }

                        if ui.link("Fetch a http request").clicked() {
                            data.rest.pre_request_script = data.rest.pre_request_script.clone().add(
                                r#"let request = {
    "method":"post",
    "url":"http://www.httpbin.org/post",
    "headers":[{
        "name":"name",
        "value":"value"
    }],
    "body":"body"
}
let response = await poster.fetch(request);
console.log(response)"#)
                        }
                    });
                });
            egui::SidePanel::left("pre_request_left")
                .resizable(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| {
                    ui.push_id("pre_request_script", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut data.rest.pre_request_script)
                                        .font(egui::TextStyle::Monospace) // for cursor height
                                        .code_editor()
                                        .desired_rows(10)
                                        .lock_focus(true)
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter),
                                );
                            });
                    });
                });
        });
        self.test_script_windows.set_and_render(ui, operation);
    }
}
