use std::collections::HashMap;

use egui::*;
use egui_snarl::Snarl;

use crate::nodes::{
    base_node::{BaseNode, InputOutputType, NodeCategory},
    node_registry::NodeRegistry,
};

#[derive(Default)]
pub struct LeftPanel {
    search: String,
    info_modal: Option<String>,
}

impl LeftPanel {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            info_modal: None,
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
        &mut self,
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
                            if info_icon_button(ui)
                                .on_hover_text("Informations")
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.info_modal = Some(node.to_string());
                            }
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

                            ui.add_space(6.0);

                        });
                    });
                });
            });

        if self.info_modal.as_deref() == Some(node) {
            self.show_info_modal(ui, node, node_registry);
        }
    }

    /// Renders the information modal for `node`, reading its description and pin
    /// types from the registry's template instance.
    fn show_info_modal(&mut self, ui: &mut egui::Ui, node: &str, node_registry: &NodeRegistry) {
        let Some(template) = node_registry.nodes.get(node) else {
            self.info_modal = None;
            return;
        };

        let description = template.informations().description;
        let inputs_summary = pins_summary(template.mapping_input(), template.inputs_count());
        let outputs_summary = pins_summary(template.mapping_output(), template.outputs_count());

        let modal =
            egui::Modal::new(egui::Id::new(("node_info_modal", node))).show(ui.ctx(), |ui| {
                ui.set_max_width(360.0);
                ui.heading(node);
                ui.separator();

                ui.label(RichText::new("Description").strong());
                ui.label(description);
                ui.add_space(8.0);

                ui.label(RichText::new("Input").strong());
                ui.label(inputs_summary);
                ui.add_space(8.0);

                ui.label(RichText::new("Output").strong());
                ui.label(outputs_summary);
                ui.add_space(8.0);

                ui.separator();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        ui.close();
                    }
                });
            });

        if modal.should_close() {
            self.info_modal = None;
        }
    }
}

/// Draws a small circled "i" info button. Painted by hand rather than using a
/// glyph so it renders consistently regardless of the bundled fonts.
fn info_icon_button(ui: &mut egui::Ui) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let center = rect.center();
        let radius = rect.width() * 0.5 - 1.0;
        let color = visuals.fg_stroke.color;
        let painter = ui.painter();
        painter.circle_stroke(center, radius, Stroke::new(1.4, color));
        painter.text(
            center,
            Align2::CENTER_CENTER,
            "i",
            FontId::proportional(12.0),
            color,
        );
    }
    response
}

/// Formats a node's pin types as one `[index] Type` line per pin, derived from
/// the node's input/output mapping. Returns `"—"` when the node has no pins.
fn pins_summary(mapping: Option<HashMap<usize, InputOutputType>>, count: usize) -> String {
    if count == 0 {
        return "—".to_string();
    }
    let map = mapping.unwrap_or_default();
    (0..count)
        .map(|i| {
            let ty = map.get(&i).map(|t| t.to_string()).unwrap_or("?");
            // Number the pins (1-based) only when there is more than one, so a
            // single pin reads simply as its type for non-technical users.
            if count > 1 {
                format!("{}. {ty}", i + 1)
            } else {
                ty.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
