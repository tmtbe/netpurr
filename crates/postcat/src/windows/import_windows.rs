use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use deno_core::error::AnyError;
use egui::{Direction, Layout, Ui};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::data::config_data::ConfigData;
use crate::data::export::{Export, ExportType};
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};
use crate::utils;

#[derive(Default)]
pub struct ImportWindows {
    open: bool,
    select_type: ImportType,
    raw: String,
    picked_path: Option<PathBuf>,
}

#[derive(EnumIter, EnumString, Display, PartialEq, Clone)]
enum ImportType {
    File,
    Raw,
}

impl Default for ImportType {
    fn default() -> Self {
        ImportType::File
    }
}
impl Window for ImportWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("IMPORT")
            .modal(true)
            .max_width(500.0)
            .max_height(400.0)
    }

    fn set_open(&mut self, open: bool) {
        self.open = open
    }

    fn get_open(&self) -> bool {
        self.open
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        config_data: &mut ConfigData,
        workspace_data: &mut WorkspaceData,
        operation: Operation,
    ) {
        ui.horizontal(|ui| {
            for import_type in ImportType::iter() {
                ui.selectable_value(
                    &mut self.select_type,
                    import_type.clone(),
                    import_type.to_string(),
                );
            }
        });
        ui.separator();
        match self.select_type {
            ImportType::File => {
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    if ui
                        .button("Drag and drop Postcat data or any of the formats below")
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if path.is_dir() {
                                operation
                                    .add_error_toast("Plural files or folder are not supported");
                            } else {
                                self.picked_path = Some(path.clone());
                            }
                        }
                    }
                });
                self.preview_files_being_dropped(ui.ctx());
                // Collect dropped files:
                ui.ctx().input(|i| {
                    if !i.raw.dropped_files.is_empty() {
                        if i.raw.dropped_files.len() > 1 {
                            operation.add_error_toast("Plural files or folder are not supported");
                        } else {
                            i.raw.dropped_files.clone().first().map(|d| {
                                d.path.clone().map(|path| {
                                    if path.is_dir() {
                                        operation.add_error_toast(
                                            "Plural files or folder are not supported",
                                        );
                                    } else {
                                        self.picked_path = Some(path.clone());
                                    }
                                });
                            });
                        }
                    };
                });
                match self.process_file(workspace_data, &operation) {
                    Ok(_) => {}
                    Err(e) => {
                        operation.add_error_toast(e.to_string());
                    }
                }
            }
            ImportType::Raw => {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        utils::text_edit_multiline_justify(ui, &mut self.raw);
                        if ui.button("Continue").clicked() {
                            match self.process_raw(self.raw.clone(), workspace_data, &operation) {
                                Ok(_) => {}
                                Err(e) => operation.add_error_toast(e.to_string()),
                            }
                        }
                    });
            }
        }
    }
}

impl ImportWindows {
    fn process_raw(
        &mut self,
        content: String,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
    ) -> Result<(), AnyError> {
        let export_result: Result<Export, serde_json::Error> =
            serde_json::from_str(content.as_str());
        match export_result {
            Ok(export) => match export.export_type {
                ExportType::Collection => {
                    export.collection.map(|c| {
                        let new_name = workspace_data.import_collection(c);
                        operation.add_success_toast(format!(
                            "Import collections `{}` success.",
                            new_name
                        ));
                    });
                }
                ExportType::Request => {}
                ExportType::Environment => {}
                ExportType::None => {
                    operation.add_error_toast("Error while importing: format not recognized");
                }
            },
            Err(_) => {
                operation.add_error_toast("Error while importing: format not recognized");
            }
        }
        Ok(())
    }
    fn process_file(
        &mut self,
        workspace_data: &mut WorkspaceData,
        operation: &Operation,
    ) -> Result<(), AnyError> {
        if let Some(path) = &self.picked_path {
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            self.process_raw(content, workspace_data, operation)?;
        }
        self.picked_path = None;
        Ok(())
    }
    fn preview_files_being_dropped(&self, ctx: &egui::Context) {
        use egui::*;
        use std::fmt::Write as _;

        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Dropping files:\n".to_owned();
                if i.raw.hovered_files.len() > 1 {
                    text = "Plural files or folder are not supported".to_owned();
                } else {
                    let file_option = i.raw.hovered_files.first();
                    match file_option {
                        None => {}
                        Some(file) => {
                            if let Some(path) = &file.path {
                                write!(text, "\n{}", path.display()).ok();
                            } else if !file.mime.is_empty() {
                                write!(text, "\n{}", file.mime).ok();
                            } else {
                                text += "\n???";
                            }
                        }
                    }
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
    }
}
