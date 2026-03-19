use eframe::egui;
use crate::gui::{components, theme};

pub fn render(ui: &mut egui::Ui, peer_count: usize, is_ops: &mut bool) {
    ui.heading("⛓️ Dashboard");
    ui.add_space(theme::SPACING_MAJOR);
    
    components::render_network_card(ui, peer_count);
    ui.add_space(theme::SPACING_MAJOR);
    
    ui.group(|ui| {
        ui.label("Control Center");
        if ui.button(if *is_ops { "Pause Node" } else { "Resume Node" }).clicked() {
            *is_ops = !*is_ops;
        }
        components::render_status_badge(ui, *is_ops);
    });
}
