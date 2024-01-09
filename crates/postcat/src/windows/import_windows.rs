use egui::Ui;

use crate::data::config_data::ConfigData;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;
use crate::operation::windows::{Window, WindowSetting};

pub struct ImportWindows {
    open: bool,
}

impl Window for ImportWindows {
    fn window_setting(&self) -> WindowSetting {
        WindowSetting::new("IMPORT")
            .modal(true)
            .min_width(500.0)
            .min_height(400.0)
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
    }
}
