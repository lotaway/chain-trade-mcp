pub mod state;
pub mod theme;
pub mod components;
pub mod views;

use eframe::egui;
pub use state::NodeState;
use state::NavigationPage;

pub struct NodeGuiApp {
    state: NodeState,
    is_operational: bool,
    navigation_stack: Vec<NavigationPage>,
}

impl NodeGuiApp {
    pub fn new(state: NodeState) -> Self {
        Self {
            state,
            is_operational: true,
            navigation_stack: vec![NavigationPage::Dashboard],
        }
    }

    fn navigate_to(&mut self, page: NavigationPage) {
        if self.navigation_stack.last() != Some(&page) {
            self.navigation_stack.push(page);
        }
    }

    fn go_back(&mut self) {
        if self.navigation_stack.len() > 1 {
            self.navigation_stack.pop();
        }
    }
}

impl eframe::App for NodeGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_global_style(ctx);
        ctx.request_repaint();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_navigation_bar(ui);
            ui.separator();
            ui.add_space(theme::SPACING_MAJOR);
            
            self.render_active_view(ui);
        });
    }
}

impl NodeGuiApp {
    fn render_navigation_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.navigation_stack.len() > 1 && ui.button("🔙").clicked() {
                self.go_back();
            }
            
            let current = self.navigation_stack.last().cloned().unwrap_or(NavigationPage::Dashboard);
            
            if ui.selectable_label(current == NavigationPage::Dashboard, "📊 Dash").clicked() {
                self.navigate_to(NavigationPage::Dashboard);
            }
            if ui.selectable_label(current == NavigationPage::NetworkExplorer, "🌐 Net").clicked() {
                self.navigate_to(NavigationPage::NetworkExplorer);
            }
        });
    }

    fn render_active_view(&mut self, ui: &mut egui::Ui) {
        let (peer_count, addresses) = self.state.get_summary();
        let current_page = self.navigation_stack.last().cloned().unwrap_or(NavigationPage::Dashboard);

        match current_page {
            NavigationPage::Dashboard => {
                views::dashboard::render(ui, peer_count, &mut self.is_operational);
            }
            NavigationPage::NetworkExplorer => {
                let mut next_page = None;
                views::explorer::render(ui, &addresses, |p| next_page = Some(p));
                if let Some(p) = next_page {
                    self.navigate_to(p);
                }
            }
            NavigationPage::PeerDetails(addr) => {
                views::details::render(ui, addr);
            }
        }
    }
}

pub fn run_gui(state: NodeState) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Blockchain Node Monitor",
        options,
        Box::new(|_cc| Ok(Box::new(NodeGuiApp::new(state)))),
    )
}
