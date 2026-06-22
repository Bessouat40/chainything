use egui::*;
use crate::types::ProcessorDef;

pub struct PanelLeft {
    pub processors: Vec<ProcessorDef>,
    pub search: String,
}

impl PanelLeft {
    pub fn new(processors: Vec<ProcessorDef>) -> Self {
        Self { processors, search: String::new() }
    }

    pub fn show(&mut self, ctx: &mut egui::Ui, dragging_processor: &mut Option<ProcessorDef>) -> Option<String> {
        let mut to_add: Option<String> = None;

        egui::Panel::left("panel_left")
            .resizable(false)
            .exact_size(220.0)
            .frame(Frame {
                fill: Color32::from_rgb(18, 18, 24),
                inner_margin: Margin::same(12),
                ..Default::default()
            })
            .show_inside(ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(Color32::from_rgb(220, 220, 235));

                ui.add_space(8.0);
                ui.label(RichText::new("NODES").size(10.0).color(Color32::from_rgb(100, 100, 130)).strong());
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

                ScrollArea::vertical().show(ui, |ui| {
                    let search_lower = self.search.to_lowercase();

                    // Group by kind
                    let pipeline_nodes: Vec<_> = self.processors.iter()
                        .filter(|p| matches!(p.kind, crate::types::NodeKind::Pipeline))
                        .filter(|p| search_lower.is_empty() || p.label.to_lowercase().contains(&search_lower))
                        .collect();

                    let ui_nodes: Vec<_> = self.processors.iter()
                        .filter(|p| matches!(p.kind, crate::types::NodeKind::Ui))
                        .filter(|p| search_lower.is_empty() || p.label.to_lowercase().contains(&search_lower))
                        .collect();

                    if !pipeline_nodes.is_empty() {
                        ui.label(RichText::new("PIPELINE").size(9.0).color(Color32::from_rgb(80, 80, 110)));
                        ui.add_space(4.0);
                        for proc in pipeline_nodes {
                            if let Some(added) = show_node_card(ui, proc, dragging_processor) {
                                to_add = Some(added);
                            }
                            ui.add_space(4.0);
                        }
                    }

                    if !ui_nodes.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new("UI ONLY").size(9.0).color(Color32::from_rgb(80, 80, 110)));
                        ui.add_space(4.0);
                        for proc in ui_nodes {
                            if let Some(added) = show_node_card(ui, proc, dragging_processor) {
                                to_add = Some(added);
                            }
                            ui.add_space(4.0);
                        }
                    }
                });
            });

        to_add
    }
}

fn show_node_card(
    ui: &mut Ui,
    proc: &ProcessorDef,
    dragging_processor: &mut Option<ProcessorDef>,
) -> Option<String> {
    let mut to_add = None;

    Frame {
        fill: Color32::from_rgb(28, 28, 38),
        corner_radius: CornerRadius::same(6),
        inner_margin: Margin::same(8),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(4.0, 32.0), Sense::hover());
            ui.painter().rect_filled(rect, CornerRadius::same(2), proc.color32());
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(&proc.label).size(11.0).strong().color(Color32::from_rgb(210, 210, 230)));
                ui.label(RichText::new(&proc.description).size(9.0).color(Color32::from_rgb(120, 120, 150)));
            });
        });

        ui.add_space(5.0);

        let btn = ui.add_sized(
            [ui.available_width(), 20.0],
            Button::new(RichText::new("+ Add").size(10.0).color(Color32::from_rgb(160, 200, 255)))
                .fill(Color32::from_rgb(35, 35, 55))
                .corner_radius(4u8),
        );

        if btn.clicked() {
            to_add = Some(proc.type_name.clone());
        }
        if btn.drag_started() {
            *dragging_processor = Some(proc.clone());
        }
    });

    to_add
}