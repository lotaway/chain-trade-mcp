use eframe::egui;
use std::net::SocketAddr;

pub fn render_peer_item(ui: &mut egui::Ui, addr: SocketAddr, on_click: impl FnOnce()) {
    ui.horizontal(|ui| {
        if ui.button("🔍").on_hover_text("Peer Details").clicked() {
            on_click();
        }
        ui.label(addr.to_string());
    });
}
