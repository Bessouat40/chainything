use egui::*;
use crate::node_renderer::base::NodeRenderer;
use crate::types::{Node, ProcessorDef};

pub struct TextInputRenderer;

impl NodeRenderer for TextInputRenderer {
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
        let scaled_width = 200.0 * zoom;
        let scaled_height = 120.0 * zoom;
        let node_rect = Rect::from_min_size(node_pos, Vec2::new(scaled_width, scaled_height));

        // Shadow
        painter.rect_filled(
            node_rect.translate(Vec2::new(3.0 * zoom, 3.0 * zoom)),
            CornerRadius::same(8),
            Color32::from_black_alpha(80),
        );

        // Body
        painter.rect_filled(node_rect, CornerRadius::same(8), Color32::from_rgb(28, 28, 42));

        // Header
        let header_rect = Rect::from_min_size(node_pos, Vec2::new(scaled_width, 32.0 * zoom));
        painter.rect_filled(header_rect, CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 }, def.color32());

        painter.text(
            header_rect.center(),
            Align2::CENTER_CENTER,
            &node.label,
            FontId::proportional(12.0 * zoom),
            Color32::WHITE,
        );

        // Drag header
        let _header_resp = ui.allocate_rect(header_rect, Sense::click_and_drag());

        // Large text area
        let content_rect = Rect::from_min_max(
            Pos2::new(node_pos.x + 8.0 * zoom, header_rect.max.y + 8.0 * zoom),
            Pos2::new(node_pos.x + scaled_width - 8.0 * zoom, node_rect.max.y - 8.0 * zoom),
        );

        let mut text = node.inputs.get(0).map(|i| i.literal_value.clone()).unwrap_or_default();
        let _resp = ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.add(
                TextEdit::multiline(&mut text)
                    .font(FontId::proportional(10.0 * zoom))
                    .desired_width(f32::INFINITY)
                    .desired_rows(3),
            )
        });

        // Output slot (right side)
        let out_center = Pos2::new(node_pos.x + scaled_width, node_pos.y + scaled_height / 2.0);
        painter.circle_filled(out_center, 6.0 * zoom, Color32::from_rgb(100, 160, 255));
        painter.circle_stroke(out_center, 6.0 * zoom, Stroke::new(1.5 * zoom, Color32::from_rgb(160, 200, 255)));

        let _out_resp = ui.allocate_rect(Rect::from_center_size(out_center, Vec2::splat(14.0 * zoom)), Sense::click());

        if let Some(ref output_def) = def.output {
            painter.text(
                Pos2::new(node_pos.x + scaled_width - 14.0 * zoom, out_center.y),
                Align2::RIGHT_CENTER,
                &output_def.type_name,
                FontId::proportional(9.0 * zoom),
                Color32::from_rgb(160, 200, 255),
            );
        }
    }
}
