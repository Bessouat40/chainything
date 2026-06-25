use egui::*;

#[derive(Default)]
pub struct BottomPanel {}

impl BottomPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .frame(egui::Frame::default().fill(ui.style().visuals.panel_fill).inner_margin(Margin::same(12)))
            .show_inside(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let run_btn = Button::new(
                        RichText::new("▶ RUN")
                            .size(14.0)
                            .color(Color32::WHITE)
                            .strong()
                    )
                    .fill(Color32::from_rgb(34, 197, 94))
                    .corner_radius(8.0)
                    .min_size(Vec2::new(130.0, 38.0));

                    if ui.add(run_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        println!("Executing DAG...");
                    }
                });
            });
    }
}