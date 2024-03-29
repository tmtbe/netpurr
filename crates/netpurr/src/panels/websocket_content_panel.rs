use egui::{Ui, Widget};
use strum::IntoEnumIterator;

use netpurr_core::data::websocket::MessageType;
use netpurr_core::data::workspace_data::WorkspaceData;

use crate::operation::operation::Operation;
use crate::panels::{HORIZONTAL_GAP, VERTICAL_GAP};
use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct WebsocketContentPanel {}

impl WebsocketContentPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &Operation,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        let mut crt = workspace_data.must_get_crt(crt_id.clone());
        ui.horizontal(|ui| {
            ui.add_space(HORIZONTAL_GAP);
            for x in MessageType::iter() {
                crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                    ui.selectable_value(
                        &mut crt.record.must_get_mut_websocket().select_message_type,
                        x.clone(),
                        x.to_string(),
                    );
                });
            }
        });
        ui.add_space(VERTICAL_GAP);
        match crt.record.must_get_mut_websocket().select_message_type {
            MessageType::Text => {}
            MessageType::Binary => {}
        }
        ui.horizontal(|ui| {
            egui::SidePanel::right("websocket_content_right_".to_string())
                .resizable(true)
                .min_width(100.0)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    let connected = crt.record.must_get_websocket().connected();
                    ui.add_enabled_ui(connected, |ui| {
                        if ui.button("Send").clicked() {
                            if let Some(session) = &crt.record.must_get_websocket().session {
                                session.send_message(
                                    crt.record.must_get_websocket().select_message_type.clone(),
                                    crt.record.must_get_websocket().retain_content.clone(),
                                )
                            }
                        }
                    });
                });
            egui::SidePanel::left("websocket_content_left_".to_string())
                .resizable(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| {
                    ui.push_id("websocket_content", |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .min_scrolled_height(300.0)
                            .show(ui, |ui| {
                                crt = workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
                                    HighlightTemplateSinglelineBuilder::default()
                                        .multiline()
                                        .envs(envs)
                                        .all_space(true)
                                        .build(
                                            "request_body".to_string(),
                                            &mut crt.record.must_get_mut_websocket().retain_content,
                                        )
                                        .ui(ui);
                                });
                            });
                    });
                });
        });
    }
}
