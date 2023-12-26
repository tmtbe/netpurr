use crate::data::AppData;
use crate::panels::central_panel::MyCentralPanel;
use crate::panels::left_panel::MyLeftPanel;
use crate::panels::{DataView, HORIZONTAL_GAP, VERTICAL_GAP};

#[derive(Default)]
pub struct App {
    left_panel: MyLeftPanel,
    central_panel: MyCentralPanel,
    app_data: AppData,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        let mut app = App::default();
        app.app_data.load_all();
        app
    }

    pub fn configure_fonts(ctx: &egui::Context) -> Option<()> {
        let font_file = Self::find_cjk_font()?;
        let font_name = font_file.split('/').last()?.split('.').next()?.to_string();
        let font_file_bytes = std::fs::read(font_file).ok()?;

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
            ui.set_enabled(!self.app_data.get_ui_lock());
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
            ui.set_enabled(!self.app_data.get_ui_lock());
            self.left_panel.set_and_render(ui, &mut self.app_data, 0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.app_data.get_ui_lock());
            self.central_panel.set_and_render(ui, &mut self.app_data, 0);
        });
    }
}
