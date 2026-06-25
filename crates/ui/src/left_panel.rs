use egui::*;

#[derive(Default)]
pub struct LeftPanel {
    search: String,
}

impl LeftPanel {
    pub fn new() -> Self {
        Self {
            search: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("left_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("NODES LIBRARY")
                        .size(10.0)
                        .color(Color32::from_rgb(100, 100, 130))
                        .strong(),
                );
                ui.add_space(8.0);

                ui.add(
                    TextEdit::singleline(&mut self.search)
                        .hint_text("Search...")
                        .desired_width(f32::INFINITY)
                        .font(TextStyle::Small),
                );
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);
            });
    }
}
