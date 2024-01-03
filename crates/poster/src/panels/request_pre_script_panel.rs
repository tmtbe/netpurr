use egui::Ui;
use log::error;

use crate::data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::script::script::Context;

#[derive(Default)]
pub struct RequestPreScriptPanel {}

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
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    if ui.hyperlink("Test").clicked() {
                        let js = data.rest.pre_request_script.clone();
                        let static_js: &'static str = Box::leak(js.into_boxed_str());
                        let result = operation.script_runtime().run(
                            static_js,
                            Context {
                                request: data.rest.request.clone(),
                                envs,
                            },
                        );
                        match result {
                            Ok(c) => {}
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        }
                    }
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
    }
}
