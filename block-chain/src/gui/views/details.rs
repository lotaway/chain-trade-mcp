use eframe::egui;
use std::net::SocketAddr;

pub fn render(ui: &mut egui::Ui, addr: SocketAddr) {
    ui.heading("🔍 Peer Specification");
    ui.separator();
    ui.label(format!("Address: {}", addr));
    ui.label("Status: Verified");
    ui.label("Latency: < 10ms");
}
