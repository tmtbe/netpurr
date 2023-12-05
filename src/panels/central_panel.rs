use crate::data::AppData;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::panels::editor_panel::EditorPanel;

#[derive(Default)]
pub struct MyCentralPanel {
    editor_panel: EditorPanel,
}

#[derive(PartialEq,Eq,Clone)]
enum PanelEnum{
    RequestId(Option<String>)
}
impl Default for PanelEnum{
    fn default() -> Self {
        PanelEnum::RequestId(None)
    }
}

impl DataView for MyCentralPanel {
    type CursorType = i32;
    fn set_and_render(&mut self,app_data: &mut AppData, cursor: Self::CursorType, ui: &mut egui::Ui){
        ui.horizontal(|ui| {
            for request_data in &app_data.central_request_data_list.data_list {
                let mut head_text = request_data.rest.request.method.to_string() +" "+ &*request_data.rest.request.url;
                if request_data.rest.request.url==""{
                    head_text = head_text + "Untitled Request";
                }
                ui.selectable_value(&mut app_data.central_request_data_list.select_id, Some(request_data.id.clone()),head_text);
            }
            if ui.button("+").clicked() {
                app_data.central_request_data_list.add_new()
            }
            if ui.button("...").clicked() {}
        });
        ui.separator();
        ui.add_space(HORIZONTAL_GAP);
        match &app_data.central_request_data_list.select_id {
            Some(request_id) => {
                self.editor_panel.set_and_render(app_data, request_id.clone(), ui);
            }
            _ => {}
        }
    }
}