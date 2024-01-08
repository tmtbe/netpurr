use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use egui::Event;
use egui_toast::{Toast, ToastKind, ToastOptions};
use log::info;
use poll_promise::Promise;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::central_panel::MyCentralPanel;
use crate::panels::left_panel::MyLeftPanel;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::windows::workspace_windows::WorkspaceWindows;

#[derive(Default)]
pub struct App {
    left_panel: MyLeftPanel,
    central_panel: MyCentralPanel,
    workspace_data: WorkspaceData,
    config_data: ConfigData,
    operation: Operation,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    current_workspace: String,
    workspace_windows: WorkspaceWindows,
    sync_promise: Option<Promise<rustygit::types::Result<()>>>,
    auto_save_time: u64,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        &cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing.x = 7.0;
            s.spacing.item_spacing.y = 7.0;
        });
        let mut app = App::default();
        app.config_data = ConfigData::load();
        app.workspace_data
            .load_all(app.config_data.select_workspace().to_string());
        app
    }

    pub fn configure_fonts(ctx: &egui::Context) -> Option<()> {
        let font_file = Self::find_cjk_font()?;
        let font_name = font_file.split('/').last()?.split('.').next()?.to_string();
        let font_file_bytes = std::fs::read(font_file).ok()?;

        let mut font_data = egui::FontData::from_owned(font_file_bytes);
        font_data.tweak.baseline_offset_factor = 0.2;

        let mut font_def = eframe::egui::FontDefinitions::default();
        font_def.font_data.insert(font_name.to_string(), font_data);

        let font_family = eframe::epaint::FontFamily::Proportional;
        font_def
            .families
            .get_mut(&font_family)?
            .insert(0, font_name.clone());

        let font_family = eframe::epaint::FontFamily::Monospace;
        font_def
            .families
            .get_mut(&font_family)?
            .insert(0, font_name.clone());

        ctx.set_fonts(font_def);
        Some(())
    }

    fn find_cjk_font() -> Option<String> {
        #[cfg(unix)]
        {
            use std::process::Command;
            // linux/macOS command: fc-list
            let output = Command::new("sh").arg("-c").arg("fc-list").output().ok()?;
            let stdout = std::str::from_utf8(&output.stdout).ok()?;
            #[cfg(target_os = "macos")]
            let font_line = stdout
                .lines()
                .find(|line| line.contains("Regular") && line.contains("Hiragino Sans GB"))
                .unwrap_or("/System/LibrarFonts/Hiragino Sans GB.ttc");
            #[cfg(target_os = "linux")]
            let font_line = stdout
                .lines()
                .find(|line| line.contains("Regular") && line.contains("CJK"))
                .unwrap_or("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc");

            let font_path = font_line.split(':').next()?.trim();
            Some(font_path.to_string())
        }
        #[cfg(windows)]
        {
            use std::path::PathBuf;
            let font_file = {
                // c:/Windows/Fonts/msyh.ttc
                let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok()?);
                font_path.push("Fonts");
                font_path.push("msyh.ttc");
                font_path.to_str()?.to_string().replace("\\", "/")
            };
            Some(font_file)
        }
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_enabled_ui(!self.operation.get_ui_lock(), |ui| {
                // The top panel is often a good place for a menu bar:
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New...").clicked() {}
                        if ui.button("Import...").clicked() {}
                        if ui.button("Exit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("View", |ui| {
                        if ui.button("Zoom In").clicked() {}
                        if ui.button("Zoom Out").clicked() {}
                    });
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
                ui.add_space(HORIZONTAL_GAP);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("New").clicked() {}
                        ui.add_space(HORIZONTAL_GAP);
                        if ui.button("Import").clicked() {}
                        ui.add_space(HORIZONTAL_GAP);
                        self.current_workspace = self.config_data.select_workspace().to_string();
                        egui::ComboBox::from_id_source("workspace")
                            .selected_text(
                                "Select workspace: ".to_string() + self.current_workspace.as_str(),
                            )
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                if ui.button("⚙ Manage Workspace").clicked() {
                                    self.workspace_windows.open(&mut self.config_data);
                                }
                                for (name, _) in self.config_data.workspaces().iter() {
                                    ui.selectable_value(
                                        &mut self.current_workspace,
                                        name.to_string(),
                                        name.to_string(),
                                    );
                                }
                            });
                        let select_workspace = self.config_data.select_workspace().to_string();
                        if let Some(workspace) = self
                            .config_data
                            .mut_workspaces()
                            .get_mut(select_workspace.as_str())
                        {
                            if workspace.if_enable_git() && workspace.if_enable_git() {
                                if self.sync_promise.is_some() {
                                    ui.add_enabled_ui(false, |ui| ui.button("🔄"));
                                } else {
                                    if ui.button("🔄").clicked() {
                                        self.sync_promise = Some(
                                            self.workspace_windows
                                                .git_sync_promise(workspace.path.clone()),
                                        );
                                    }
                                }
                            }
                        }
                        match &self.sync_promise {
                            None => {}
                            Some(result) => match result.ready() {
                                None => {
                                    ui.ctx().request_repaint();
                                }
                                Some(result) => {
                                    if result.is_ok() {
                                        self.operation.toasts().add(Toast {
                                            text: "Sync Success!".into(),
                                            kind: ToastKind::Success,
                                            options: ToastOptions::default()
                                                .duration_in_seconds(5.0)
                                                .show_progress(true),
                                        });
                                    } else {
                                        self.operation.toasts().add(Toast {
                                            text: "Sync Failed!".into(),
                                            kind: ToastKind::Error,
                                            options: ToastOptions::default()
                                                .duration_in_seconds(5.0)
                                                .show_progress(true),
                                        });
                                    }
                                    self.sync_promise = None;
                                    self.workspace_data
                                        .reload_data(self.current_workspace.clone());
                                }
                            },
                        }
                        if self.current_workspace != self.config_data.select_workspace() {
                            self.config_data
                                .set_select_workspace(self.current_workspace.clone());
                            self.workspace_data = WorkspaceData::default();
                            self.workspace_data.load_all(self.current_workspace.clone());
                        }
                    });
                });
                ui.add_space(HORIZONTAL_GAP);
            });
            self.workspace_windows
                .set_and_render(ui, &mut self.operation, &mut self.config_data);
        });
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.add_enabled_ui(!self.operation.get_ui_lock(), |ui| {
                self.left_panel.set_and_render(
                    ui,
                    &mut self.operation,
                    &mut self.workspace_data,
                    0,
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.operation.get_ui_lock(), |ui| {
                self.central_panel.set_and_render(
                    ui,
                    &mut self.operation,
                    &mut self.workspace_data,
                    0,
                );
            });
        });

        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.allowed_to_close {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_confirmation_dialog = true;
            }
        }
        // auto save
        if ctx.input(|i| {
            i.events
                .iter()
                .filter(|event| match event {
                    Event::Key { .. } => true,
                    _ => false,
                })
                .count()
                > 0
        }) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            if now - self.auto_save_time > 5 {
                self.auto_save_time = now;
                self.workspace_data.auto_save_crd();
                info!("auto save");
            }
        }
        if self.show_confirmation_dialog {
            self.operation.lock_ui("Quit".to_string(), true);
            egui::Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("No").clicked() {
                            self.show_confirmation_dialog = false;
                            self.allowed_to_close = false;
                            self.operation.lock_ui("Quit".to_string(), false);
                        }

                        if ui.button("Yes").clicked() {
                            self.show_confirmation_dialog = false;
                            self.allowed_to_close = true;
                            self.workspace_data.auto_save_crd();
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
        }
        self.operation.toasts().show(ctx);
    }
}
