mod app;
mod node_editor;
mod node_renderer;
mod panel_left;
mod types;

use app::ChainythingApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Chainything"),
        ..Default::default()
    };

    eframe::run_native(
        "Chainything",
        native_options,
        Box::new(|cc| Ok(Box::new(ChainythingApp::new(cc)))),
    )
}