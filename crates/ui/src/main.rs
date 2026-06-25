use eframe::egui;
mod app;
mod bottom_panel;
mod dag_layout;
mod left_panel;
mod nodes;

use app::ChainythingApp;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Chainything"),
        ..Default::default()
    };
    eframe::run_native("MyApp",
    native_options,
    Box::new(|cc| {

        egui_extras::install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(ChainythingApp::new(cc)))
    }))
}

