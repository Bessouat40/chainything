use egui::*;
use egui_snarl::Snarl;

use crate::nodes::{
    base_node::{BaseNode, NodeCategory},
    node_registry::NodeRegistry,
};

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

    fn add_node(
        &self,
        node_name: &str,
        node_registry: &NodeRegistry,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
        ctx: &egui::Context,
    ) {
        if let Some(node) = node_registry.create_node(node_name) {
            let center_pos = ctx.content_rect().center();
            snarl.insert_node(center_pos, node);
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
        node_registry: &NodeRegistry,
    ) {
        egui::Panel::left("left_panel")
            .resizable(false)
            .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(Margin::same(12)))
            .show_inside(ui, |ui| {
                ui.label(
                    RichText::new("NODES LIBRARY")
                        .size(11.0)
                        .color(Color32::from_rgb(140, 145, 155))
                        .strong(),
                );
                ui.add_space(8.0);

                ui.add(
                    TextEdit::singleline(&mut self.search)
                        .hint_text("🔍 Search nodes...")
                        .desired_width(f32::INFINITY)
                        .margin(Margin::same(6)),
                );

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                let search_lower = self.search.to_lowercase();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for category in NodeCategory::ALL {
                        // Collect the nodes of this category that match the search,
                        // sorted by name so the order is stable (the registry is a
                        // HashMap with non-deterministic iteration order).
                        let mut names: Vec<&String> = node_registry
                            .nodes
                            .iter()
                            .filter(|(name, node)| {
                                node.category() == category
                                    && (search_lower.is_empty()
                                        || name.to_lowercase().contains(&search_lower))
                            })
                            .map(|(name, _)| name)
                            .collect();
                        names.sort();

                        if names.is_empty() {
                            continue;
                        }

                        let header =
                            RichText::new(format!("{} ({})", category.label(), names.len()))
                                .size(11.0)
                                .color(Color32::from_rgb(120, 125, 135))
                                .strong();

                        egui::CollapsingHeader::new(header)
                            // Keep sections open by default, but force-open while a
                            // search is active so matches are never hidden.
                            .default_open(true)
                            .open(if search_lower.is_empty() {
                                None
                            } else {
                                Some(true)
                            })
                            .id_salt(category.label())
                            .show_unindented(ui, |ui| {
                                ui.add_space(6.0);
                                for node in names {
                                    self.show_node_entry(ui, node, node_registry, snarl);
                                    ui.add_space(8.0);
                                }
                            });

                        ui.add_space(8.0);
                    }
                });
            });
    }

    fn show_node_entry(
        &self,
        ui: &mut egui::Ui,
        node: &str,
        node_registry: &NodeRegistry,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
    ) {
        egui::Frame::new()
            .fill(Color32::from_rgb(43, 45, 49))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 65, 70)))
            .corner_radius(8.0)
            .inner_margin(Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(
                                RichText::new(node)
                                    .size(14.0)
                                    .color(Color32::from_rgb(210, 215, 220))
                                    .strong(),
                            );
                        });
                        ui.add_space(5.0);

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let add_btn = Button::new(
                                RichText::new("+ Add")
                                    .size(12.0)
                                    .color(Color32::from_rgb(240, 242, 245))
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(70, 110, 170))
                            .corner_radius(6.0)
                            .min_size(Vec2::new(60.0, 24.0));

                            if ui
                                .add(add_btn)
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.add_node(node, node_registry, snarl, ui.ctx());
                            }
                        });
                    });
                });
            });
    }
}
