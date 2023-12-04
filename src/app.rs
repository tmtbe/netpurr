use std::cell::RefCell;
use std::rc::Rc;

use egui::Ui;

use crate::events::MailPost;
use crate::panels::{HORIZONTAL_GAP, VERTICAL_GAP, View};
use crate::panels::central_panel::MyCentralPanel;
use crate::panels::left_panel::MyLeftPanel;

#[derive(Default)]
pub struct App {
    left_panel: MyLeftPanel,
    central_panel: MyCentralPanel,
    mail_post: Rc<RefCell<MailPost>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mail_post = Rc::new(RefCell::new(MailPost::default()));
        let mut app = App::default();
        app.init(mail_post);
        app
    }
}

impl View for App {
    fn init(&mut self, mail_post: Rc<RefCell<MailPost>>) {
        self.mail_post = mail_post.clone();
        self.left_panel.init(mail_post.clone());
        self.central_panel.init(mail_post.clone());
    }

    fn render(&mut self, ui: &mut Ui, mail_post: Rc<RefCell<MailPost>>) {}
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
            self.left_panel.render(ui, self.mail_post.clone());
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel.render(ui, self.mail_post.clone());
        });
    }
}