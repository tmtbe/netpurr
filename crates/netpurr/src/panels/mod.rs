use crate::data::workspace_data::WorkspaceData;
use crate::operation::operation::Operation;

pub mod auth_panel;
pub mod bottom_panel;
pub mod central_panel;
pub mod collections_panel;
pub mod history_panel;
pub mod left_panel;
pub mod request_body_form_data_panel;
pub mod request_body_panel;
pub mod request_body_xxx_form_panel;
pub mod request_headers_panel;
pub mod request_params_panel;
pub mod request_pre_script_panel;
pub mod response_body_panel;
pub mod response_cookies_panel;
pub mod response_headers_panel;
pub mod response_log_panel;
pub mod response_panel;
pub mod rest_panel;
pub mod right_panel;
pub mod test_result_panel;
pub mod test_script_panel;
pub mod top_panel;
pub mod websocket_content_panel;
pub mod websocket_event_panel;
pub mod websocket_panel;

pub const HORIZONTAL_GAP: f32 = 8.0;
pub const VERTICAL_GAP: f32 = 2.0;

pub trait DataView {
    type CursorType;
    fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        cursor: Self::CursorType,
    );
}
