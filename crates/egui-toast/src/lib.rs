//! This crate provides a convenient interface for showing toast notifications with
//! the [egui](https://github.com/emilk/egui) library.
//!
//! For a complete example, see <https://github.com/urholaukkarinen/egui-toast/tree/main/demo>.
//!
//! # Usage
//!
//! To get started, create a `Toasts` instance in your rendering code and specify the anchor position and
//! direction for the notifications. Toast notifications will show up starting from the specified
//! anchor position and stack up in the specified direction.
//! ```
//! # use std::time::Duration;
//! use egui::Align2;
//! # use egui_toast::{Toasts, ToastKind, ToastOptions, Toast};
//! # egui_toast::__run_test_ui(|ui, ctx| {
//! let mut toasts = Toasts::new()
//!     .anchor(Align2::LEFT_TOP, (10.0, 10.0))
//!     .direction(egui::Direction::TopDown);
//!
//! toasts.add(Toast {
//!     text: "Hello, World".into(),
//!     kind: ToastKind::Info,
//!     options: ToastOptions::default()
//!         .duration_in_seconds(3.0)
//!         .show_progress(true)
//!         .show_icon(true)
//! });

//!
//! // Show all toasts
//! toasts.show(ctx);
//! # })
//! ```
//!
//! Look of the notifications can be fully customized by specifying a custom rendering function for a specific toast kind
//! with [`Toasts::custom_contents`]. [`ToastKind::Custom`] can be used if the default kinds are not sufficient.
//!
//! ```
//! # use std::time::Duration;
//! # use std::sync::Arc;
//! # use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
//! # egui_toast::__run_test_ui(|ui, ctx| {
//! const MY_CUSTOM_TOAST: u32 = 0;
//!
//! fn custom_toast_contents(ui: &mut egui::Ui, toast: &mut Toast) -> egui::Response {
//!     egui::Frame::window(ui.style()).show(ui, |ui| {
//!         ui.label(toast.text.clone());
//!     }).response
//! }
//!
//! let mut toasts = Toasts::new()
//!     .custom_contents(MY_CUSTOM_TOAST, custom_toast_contents);
//!
//! // Add a custom toast that never expires
//! toasts.add(Toast {
//!     text: "Hello, World".into(),
//!     kind: ToastKind::Custom(MY_CUSTOM_TOAST),
//!     options: ToastOptions::default(),
//! });
//!
//! # })
//! ```
//!
#![deny(clippy::all)]

mod toast;
pub use toast::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use egui::epaint::RectShape;
use egui::{
    Align2, Area, Color32, Context, Direction, Frame, Id, Order, Pos2, Response, RichText,
    Rounding, Shape, Stroke, Ui,
};

pub const INFO_COLOR: Color32 = Color32::from_rgb(0, 155, 255);
pub const WARNING_COLOR: Color32 = Color32::from_rgb(255, 212, 0);
pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 32, 0);
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(0, 255, 32);

pub type ToastContents = dyn Fn(&mut Ui, &mut Toast) -> Response + Send + Sync;

pub struct Toasts {
    id: Id,
    align: Align2,
    offset: Pos2,
    direction: Direction,
    custom_toast_contents: HashMap<ToastKind, Arc<ToastContents>>,
    /// Toasts added since the last draw call. These are moved to the
    /// egui context's memory, so you are free to recreate the [`Toasts`] instance every frame.
    added_toasts: Vec<Toast>,
}

impl Default for Toasts {
    fn default() -> Self {
        Self {
            id: Id::new("__toasts"),
            align: Align2::LEFT_TOP,
            offset: Pos2::new(10.0, 10.0),
            direction: Direction::TopDown,
            custom_toast_contents: HashMap::new(),
            added_toasts: Vec::new(),
        }
    }
}

impl Toasts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Position where the toasts show up.
    ///
    /// The toasts will start from this position and stack up
    /// in the direction specified with [`Self::direction`].
    pub fn position(mut self, position: impl Into<Pos2>) -> Self {
        self.offset = position.into();
        self
    }

    /// Anchor for the toasts.
    ///
    /// For instance, if you set this to (10.0, 10.0) and [`Align2::LEFT_TOP`],
    /// then (10.0, 10.0) will be the top-left corner of the first toast.
    pub fn anchor(mut self, anchor: Align2, offset: impl Into<Pos2>) -> Self {
        self.align = anchor;
        self.offset = offset.into();
        self
    }

    /// Direction where the toasts stack up
    pub fn direction(mut self, direction: impl Into<Direction>) -> Self {
        self.direction = direction.into();
        self
    }

    /// Can be used to specify a custom rendering function for toasts for given kind
    pub fn custom_contents(
        mut self,
        kind: impl Into<ToastKind>,
        add_contents: impl Fn(&mut Ui, &mut Toast) -> Response + Send + Sync + 'static,
    ) -> Self {
        self.custom_toast_contents
            .insert(kind.into(), Arc::new(add_contents));
        self
    }

    /// Add a new toast
    pub fn add(&mut self, toast: Toast) -> &mut Self {
        self.added_toasts.push(toast);
        self
    }

    /// Show and update all toasts
    pub fn show(&mut self, ctx: &Context) {
        let Self {
            id,
            align,
            mut offset,
            direction,
            ..
        } = *self;

        let dt = ctx.input(|i| i.unstable_dt) as f64;

        let mut toasts: Vec<Toast> = ctx.data_mut(|d| d.get_temp(id).unwrap_or_default());
        toasts.extend(std::mem::take(&mut self.added_toasts));
        toasts.retain(|toast| toast.options.ttl_sec > 0.0);

        for (i, toast) in toasts.iter_mut().enumerate() {
            let response = Area::new(id.with("toast").with(i))
                .anchor(align, offset.to_vec2())
                .order(Order::Foreground)
                .interactable(true)
                .show(ctx, |ui| {
                    if let Some(add_contents) = self.custom_toast_contents.get_mut(&toast.kind) {
                        add_contents(ui, toast)
                    } else {
                        default_toast_contents(ui, toast)
                    };
                })
                .response;

            if !response.hovered() {
                toast.options.ttl_sec -= dt;
                if toast.options.ttl_sec.is_finite() {
                    ctx.request_repaint_after(Duration::from_secs_f64(
                        toast.options.ttl_sec.max(0.0),
                    ));
                }
            }

            if toast.options.show_progress {
                ctx.request_repaint();
            }

            match direction {
                Direction::LeftToRight => {
                    offset.x += response.rect.width() + 10.0;
                }
                Direction::RightToLeft => {
                    offset.x -= response.rect.width() + 10.0;
                }
                Direction::TopDown => {
                    offset.y += response.rect.height() + 10.0;
                }
                Direction::BottomUp => {
                    offset.y -= response.rect.height() + 10.0;
                }
            }
        }

        ctx.data_mut(|d| d.insert_temp(id, toasts));
    }
}

fn default_toast_contents(ui: &mut Ui, toast: &mut Toast) -> Response {
    let inner_margin = 10.0;
    let frame = Frame::window(ui.style());
    let response = frame
        .inner_margin(inner_margin)
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (icon, color) = match toast.kind {
                    ToastKind::Warning => ("⚠", WARNING_COLOR),
                    ToastKind::Error => ("❗", ERROR_COLOR),
                    ToastKind::Success => ("✔", SUCCESS_COLOR),
                    _ => ("ℹ", INFO_COLOR),
                };

                let a = |ui: &mut Ui, toast: &mut Toast| {
                    if toast.options.show_icon {
                        ui.label(RichText::new(icon).color(color));
                    }
                };
                let b = |ui: &mut Ui, toast: &mut Toast| ui.label(toast.text.clone());
                let c = |ui: &mut Ui, toast: &mut Toast| {
                    if ui.button("🗙").clicked() {
                        toast.close();
                    }
                };

                // Draw the contents in the reverse order on right-to-left layouts
                // to keep the same look.
                if ui.layout().prefer_right_to_left() {
                    c(ui, toast);
                    b(ui, toast);
                    a(ui, toast);
                } else {
                    a(ui, toast);
                    b(ui, toast);
                    c(ui, toast);
                }
            })
        })
        .response;

    if toast.options.show_progress {
        progress_bar(ui, &response, toast);
    }

    // Draw the frame's stroke last
    let frame_shape = Shape::Rect(RectShape::stroke(
        response.rect,
        frame.rounding,
        ui.visuals().window_stroke,
    ));
    ui.painter().add(frame_shape);

    response
}

fn progress_bar(ui: &mut Ui, response: &Response, toast: &Toast) {
    let rounding = Rounding {
        nw: 0.0,
        ne: 0.0,
        ..ui.visuals().window_rounding
    };
    let mut clip_rect = response.rect;
    clip_rect.set_top(clip_rect.bottom() - 2.0);
    clip_rect.set_right(clip_rect.left() + clip_rect.width() * toast.options.progress() as f32);

    ui.painter().with_clip_rect(clip_rect).rect_filled(
        response.rect,
        rounding,
        ui.visuals().text_color(),
    );
}

pub fn __run_test_ui(mut add_contents: impl FnMut(&mut Ui, &Context)) {
    let ctx = Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            add_contents(ui, ctx);
        });
    });
}

pub fn __run_test_ui_with_toasts(mut add_contents: impl FnMut(&mut Ui, &mut Toasts)) {
    let ctx = Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut toasts = Toasts::new();
            add_contents(ui, &mut toasts);
        });
    });
}
