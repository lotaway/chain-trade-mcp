use eframe::egui;
use std::net::SocketAddr;
use crate::gui::{state, components, theme};

pub fn render(ui: &mut egui::Ui, peers: &[SocketAddr], mut navigate: impl FnMut(state::NavigationPage)) {
    ui.heading("🌐 Network Explorer");
    ui.add_space(theme::SPACING_MAJOR);
    
    egui::ScrollArea::vertical().max_height(theme::LIST_MIN_HEIGHT).show(ui, |ui| {
        for &addr in peers {
            components::render_peer_item(ui, addr, || navigate(state::NavigationPage::PeerDetails(addr)));
        }
    });
}
