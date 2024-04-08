use std::time::{SystemTime, UNIX_EPOCH};

use egui::{Context, Event, Frame, Margin, Visuals};
use log::info;
use poll_promise::Promise;

use netpurr_core::data::workspace_data::WorkspaceData;

use crate::data::config_data::ConfigData;
use crate::operation::operation::Operation;
use crate::panels::bottom_panel::BottomPanel;
use crate::panels::selected_collection_panel::SelectedCollectionPanel;
use crate::panels::selected_workspace_panel::SelectedWorkspacePanel;
use crate::panels::top_panel::TopPanel;

pub struct App {
    workspace_data: WorkspaceData,
    operation: Operation,
    config_data: ConfigData,
    selected_collection_panel: SelectedCollectionPanel,
    selected_workspace_panel: SelectedWorkspacePanel,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    current_workspace: String,
    sync_promise: Option<Promise<rustygit::types::Result<()>>>,
    auto_save_time: u64,
    top_panel: TopPanel,
    bottom_panel: BottomPanel,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        &cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing.x = 7.0;
            s.spacing.item_spacing.y = 7.0;
        });
        let mut workspace_data = WorkspaceData::default();
        let config_data = ConfigData::load();
        config_data.select_workspace().map(|workspace| {
            workspace_data.load_all(workspace);
        });
        let mut app = App {
            operation: Operation::new(workspace_data.get_cookies_manager()),
            config_data,
            selected_collection_panel: Default::default(),
            selected_workspace_panel: Default::default(),
            show_confirmation_dialog: false,
            allowed_to_close: false,
            current_workspace: "".to_string(),
            sync_promise: None,
            workspace_data,
            auto_save_time: 0,
            top_panel: Default::default(),
            bottom_panel: Default::default(),
        };
        app
    }

    pub fn configure_fonts(ctx: &Context) -> Option<()> {
        let font_name = "NotoSansSC-Regular".to_string();
        let font_file_bytes = include_bytes!("./../font/NotoSansSC-Regular.ttf").to_vec();

        let font_data = egui::FontData::from_owned(font_file_bytes);

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
            .push(font_name.clone());
        egui_phosphor::add_to_fonts(&mut font_def, egui_phosphor::Variant::Regular);
        ctx.set_fonts(font_def);
        Some(())
    }

    fn auto_save(&mut self, ctx: &Context) {
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
    }

    fn quit_dialog(&mut self, ctx: &Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.allowed_to_close {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.show_confirmation_dialog = true;
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
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.operation
            .show(ctx, &mut self.config_data, &mut self.workspace_data);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.top_panel.render(
                ui,
                &mut self.workspace_data,
                self.operation.clone(),
                &mut self.config_data,
            )
        });

        egui::CentralPanel::default()
            .frame(Frame {
                inner_margin: Margin::ZERO,
                outer_margin: Margin::ZERO,
                rounding: Default::default(),
                shadow: Default::default(),
                fill: Default::default(),
                stroke: Default::default(),
            })
            .show(ctx, |ui| {
                if self.config_data.select_collection().is_some()
                    && self.config_data.select_workspace().is_some()
                {
                    ui.add_enabled_ui(!self.operation.get_ui_lock(), |ui| {
                        self.selected_collection_panel.set_and_render(
                            ui,
                            &self.operation,
                            &mut self.workspace_data,
                            &mut self.config_data,
                        );
                    });
                } else {
                    self.selected_workspace_panel.set_and_render(
                        ui,
                        &self.operation,
                        &mut self.workspace_data,
                        &mut self.config_data,
                    );
                }
            });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.bottom_panel.render(
                ui,
                &mut self.workspace_data,
                self.operation.clone(),
                &mut self.config_data,
            )
        });

        self.auto_save(ctx);
        self.quit_dialog(ctx);
    }
}
