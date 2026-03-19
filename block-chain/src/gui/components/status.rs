use eframe::egui;
use crate::gui::theme;

pub fn render_status_badge(ui: &mut egui::Ui, is_active: bool) {
    let (icon, label, color) = if is_active {
        (theme::ICON_ACTIVE, "Active", egui::Color32::GREEN)
    } else {
        (theme::ICON_SUSPENDED, "Paused", egui::Color32::GOLD)
    };
    ui.horizontal(|ui| {
        ui.label(icon);
        ui.colored_label(color, label);
    });
}
