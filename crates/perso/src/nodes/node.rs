use egui::*;

pub struct Node {
    node_pos: Pos2,
}

impl Default for Node {

    fn default() -> Self {
        Self { node_pos: pos2(30.0, 30.0) }
    }
}

impl Node {
    pub fn new(pos: Pos2) -> Self {
        Self { node_pos: pos }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let node_size = vec2(200.0, 100.0);
        let rect = Rect::from_min_size(self.node_pos, node_size);

        let response = ui.allocate_rect(rect, Sense::drag());

        if response.dragged() {
            self.node_pos += response.drag_delta();
        }

        let color = if response.dragged() {
            Color32::from_rgb(200, 200, 200)
        } else if response.hovered() {
            Color32::from_rgb(200, 200, 200)
        } else {
            Color32::from_rgb(255, 255, 255)
        };

        ui.painter().rect_filled(
            rect,
            CornerRadius::same(8),
            color,
        );
    }
}