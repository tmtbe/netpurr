use crate::data::AppData;
use crate::panels::{DataView, HORIZONTAL_GAP, VERTICAL_GAP};
use crate::panels::central_panel::MyCentralPanel;
use crate::panels::left_panel::MyLeftPanel;

#[derive(Default)]
pub struct App {
    left_panel: MyLeftPanel,
    central_panel: MyCentralPanel,
    app_data: AppData,
}

impl App {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        let mut app = App::default();
        app.app_data.fake();
        app
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New...").clicked() {}
                    if ui.button("Import...").clicked() {}
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(VERTICAL_GAP);
                ui.menu_button("View", |ui| {
                    if ui.button("Zoom In").clicked() {}
                    if ui.button("Zoom Out").clicked() {}
                });
                ui.add_space(VERTICAL_GAP);
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
            ui.add_space(HORIZONTAL_GAP);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New").clicked() {}
                    ui.add_space(VERTICAL_GAP);
                    if ui.button("Import").clicked() {}
                });
            });
            ui.add_space(HORIZONTAL_GAP);
        });
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            self.left_panel.set_and_render(&mut self.app_data,0, ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel.set_and_render(&mut self.app_data,0, ui);
        });
    }
}