use crate::data::AppData;

pub mod central_panel;
pub mod collections_panel;
pub mod editor_panel;
pub mod history_panel;
pub mod left_panel;
pub mod params_panel;
mod reponse_panel;
mod body_panel;

pub const HORIZONTAL_GAP: f32 = 8.0;
pub const VERTICAL_GAP: f32 = 8.0;

pub trait DataView {
    type CursorType;
    fn set_and_render(
        &mut self,
        app_data: &mut AppData,
        cursor: Self::CursorType,
        ui: &mut egui::Ui,
    );
}
