use crate::dag_layout::DAGLayout;
use egui::*;

#[derive(Default)]
pub struct BottomPanel;

impl BottomPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, dag_layout: &mut DAGLayout) {
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(ui.style().visuals.panel_fill)
                    .inner_margin(Margin::same(12)),
            )
            .show_inside(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let is_running = dag_layout.is_running();
                    let run_btn = Button::new(
                        RichText::new(if is_running {
                            "⏸ RUNNING..."
                        } else {
                            "▶ RUN"
                        })
                        .size(14.0)
                        .color(Color32::WHITE)
                        .strong(),
                    )
                    .fill(if is_running {
                        Color32::from_rgb(200, 100, 100)
                    } else {
                        Color32::from_rgb(34, 197, 94)
                    })
                    .corner_radius(8.0)
                    .min_size(Vec2::new(130.0, 38.0));

                    if ui
                        .add_enabled(!is_running, run_btn)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        dag_layout.run();
                    }
                });
            });
    }
}
