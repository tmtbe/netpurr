use egui::Ui;

use crate::data::AppData;
use crate::panels::DataView;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct CollectionsPanel {}

impl DataView for CollectionsPanel {
    type CursorType = i32;

    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {}
}
