use egui::*;

#[derive(Default)]
pub struct BottomPanel {
}

impl BottomPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui | {
                    ui.add(
                        Button::new(RichText::new("▶  Run").size(13.0).color(Color32::WHITE).strong())
                                .fill(Color32::from_rgb(60, 120, 220))
                                .corner_radius(6u8)
                                .min_size(Vec2::new(120.0, 32.0)),
                    )
                });
            });
    }
}