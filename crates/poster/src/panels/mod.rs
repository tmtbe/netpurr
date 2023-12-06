use crate::data::AppData;

pub mod central_panel;
pub mod collections_panel;
pub mod history_panel;
pub mod left_panel;
mod request_headers_panel;
pub mod request_params_panel;
mod response_body_panel;
mod response_cookies_panel;
mod response_headers_panel;
mod response_panel;
pub mod rest_panel;

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
