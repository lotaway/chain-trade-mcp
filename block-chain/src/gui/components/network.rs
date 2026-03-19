use eframe::egui;

pub fn render_network_card(ui: &mut egui::Ui, peer_count: usize) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label("Network Nodes");
            ui.heading(format!("{} Peers Connected", peer_count));
        });
    });
}
