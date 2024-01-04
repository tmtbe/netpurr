use crate::data::WorkspaceData;
use crate::operation::Operation;

mod auth_panel;
pub mod central_panel;
pub mod collections_panel;
mod cookies_windows;
mod environment_windows;
pub mod history_panel;
pub mod left_panel;
mod new_collection_windows;
mod request_body_form_data_panel;
mod request_body_panel;
mod request_body_xxx_form_panel;
mod request_headers_panel;
pub mod request_params_panel;
mod request_pre_script_panel;
mod response_body_panel;
mod response_cookies_panel;
mod response_headers_panel;
mod response_panel;
pub mod rest_panel;
mod save_windows;
mod test_script_windows;
pub mod workspace_windows;

pub const HORIZONTAL_GAP: f32 = 8.0;
pub const VERTICAL_GAP: f32 = 2.0;

pub trait AlongDataView {
    type DataType;
    fn set_and_render(&mut self, ui: &mut egui::Ui, data: &mut Self::DataType);
}

pub trait DataView {
    type CursorType;
    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    );
}
