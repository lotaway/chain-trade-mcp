use eframe::egui;

pub const SPACING_MAJOR: f32 = 12.0;
pub const SPACING_MINOR: f32 = 6.0;
pub const LIST_MIN_HEIGHT: f32 = 250.0;

pub const ICON_ACTIVE: &str = "🟢";
pub const ICON_SUSPENDED: &str = "🟡";

pub fn apply_global_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.widgets.noninteractive.rounding = 4.0.into();
    ctx.set_visuals(visuals);
}
