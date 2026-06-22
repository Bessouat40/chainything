use egui::*;
use crate::node_renderer::base::NodeRenderer;
use crate::types::{Node, ProcessorDef, NodeKind};

const NODE_WIDTH: f32 = 180.0;
const SLOT_HEIGHT: f32 = 22.0;
const HEADER_HEIGHT: f32 = 32.0;

pub struct DefaultNodeRenderer;

impl NodeRenderer for DefaultNodeRenderer {
    fn render(
        &self,
        node: &Node,
        def: &ProcessorDef,
        painter: &Painter,
        ui: &mut Ui,
        _ctx: &Context,
        node_pos: Pos2,
        zoom: f32,
        _pan_offset: Vec2,
        _canvas_rect: Rect,
    ) {
        let num_inputs = node.inputs.len();
        let scaled_header = HEADER_HEIGHT * zoom;
        let scaled_slot = SLOT_HEIGHT * zoom;
        let scaled_width = NODE_WIDTH * zoom;
        let node_height = scaled_header + num_inputs as f32 * scaled_slot + 12.0 * zoom;
        let node_rect = Rect::from_min_size(node_pos, Vec2::new(scaled_width, node_height));

        // Shadow
        painter.rect_filled(
            node_rect.translate(Vec2::new(3.0 * zoom, 3.0 * zoom)),
            CornerRadius::same(8),
            Color32::from_black_alpha(80),
        );

        // Body
        painter.rect_filled(node_rect, CornerRadius::same(8), Color32::from_rgb(28, 28, 42));

        // Header
        let def_color = def.color32();
        let header_color = if matches!(node.kind, NodeKind::Ui) {
            Color32::from_rgba_premultiplied(
                def_color.r() / 2,
                def_color.g() / 2,
                def_color.b() / 2,
                220,
            )
        } else {
            def_color
        };

        let header_rect = Rect::from_min_size(node_pos, Vec2::new(scaled_width, scaled_header));
        painter.rect_filled(header_rect, CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 }, header_color);

        painter.text(
            header_rect.center(),
            Align2::CENTER_CENTER,
            &node.label,
            FontId::proportional(12.0 * zoom),
            Color32::WHITE,
        );

        // UI-only label
        if matches!(node.kind, NodeKind::Ui) {
            painter.text(
                Pos2::new(header_rect.max.x - 4.0 * zoom, header_rect.min.y + 4.0 * zoom),
                Align2::RIGHT_TOP,
                "UI",
                FontId::proportional(8.0 * zoom),
                Color32::from_rgb(255, 220, 100),
            );
        }

        // Drag header
        let header_resp = ui.allocate_rect(header_rect, Sense::click_and_drag());
        if header_resp.drag_started() {
            // Handled by node editor
        }

        // Input slots
        for (slot_idx, _) in node.inputs.iter().enumerate() {
            let slot_y = node_pos.y + scaled_header + slot_idx as f32 * scaled_slot + scaled_slot / 2.0;
            let slot_center = Pos2::new(node_pos.x, slot_y);

            let connected = node.inputs[slot_idx].source_node.is_some();
            painter.circle_filled(slot_center, 6.0 * zoom, if connected {
                Color32::from_rgb(100, 200, 120)
            } else {
                Color32::from_rgb(80, 80, 100)
            });
            painter.circle_stroke(slot_center, 6.0 * zoom, Stroke::new(1.5 * zoom, Color32::from_rgb(160, 160, 200)));

            let dot_resp = ui.allocate_rect(Rect::from_center_size(slot_center, Vec2::splat(14.0 * zoom)), Sense::click());
            if dot_resp.clicked() {
                // Handled by node editor
            }

            // Slot label
            let slot_label = def.inputs.get(slot_idx).map(|s| s.label.as_str()).unwrap_or("in");
            painter.text(
                Pos2::new(node_pos.x + 14.0 * zoom, slot_y),
                Align2::LEFT_CENTER,
                slot_label,
                FontId::proportional(10.0 * zoom),
                Color32::from_rgb(160, 160, 190),
            );

            // Literal text field
            let accepts_literal = def.inputs.get(slot_idx).map(|s| s.accepts_literal).unwrap_or(false);
            if accepts_literal && !connected {
                let field_rect = Rect::from_min_size(
                    Pos2::new(node_pos.x + 60.0 * zoom, slot_y - 9.0 * zoom),
                    Vec2::new(scaled_width - 70.0 * zoom, 18.0 * zoom),
                );
                let mut text = node.inputs[slot_idx].literal_value.clone();
                let _resp = ui.allocate_ui_at_rect(field_rect, |ui| {
                    ui.add(
                        TextEdit::singleline(&mut text)
                            .font(FontId::proportional(9.0 * zoom))
                            .desired_width(field_rect.width()),
                    )
                });
                if _resp.inner.changed() {
                    // Handled by node editor
                }
            }
        }

        // Output slot
        let out_y = node_pos.y + scaled_header + num_inputs as f32 * scaled_slot / 2.0 + scaled_slot / 2.0;
        let out_center = Pos2::new(node_pos.x + scaled_width, out_y);
        painter.circle_filled(out_center, 6.0 * zoom, Color32::from_rgb(100, 160, 255));
        painter.circle_stroke(out_center, 6.0 * zoom, Stroke::new(1.5 * zoom, Color32::from_rgb(160, 200, 255)));

        let out_resp = ui.allocate_rect(Rect::from_center_size(out_center, Vec2::splat(14.0 * zoom)), Sense::click());
        if out_resp.clicked() {
            // Handled by node editor
        }

        // Output type label
        if let Some(ref output_def) = def.output {
            painter.text(
                Pos2::new(node_pos.x + scaled_width - 14.0 * zoom, out_y),
                Align2::RIGHT_CENTER,
                &output_def.type_name,
                FontId::proportional(9.0 * zoom),
                Color32::from_rgb(160, 200, 255),
            );
        }
    }
}
