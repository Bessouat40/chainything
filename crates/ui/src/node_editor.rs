use egui::*;
use std::sync::Arc;
use crate::types::{Node, Connection, NodeInput, NodeKind, ProcessorDef, PipelineJson, NodeJson, InputJson};
use crate::node_renderer::{NodeRendererRegistry, DefaultNodeRenderer, TextInputRenderer};

pub struct NodeEditor {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub pan_offset: Vec2,
    pub zoom: f32,
    dragging_node: Option<String>,
    drag_start: Pos2,
    drag_node_start: Pos2,
    pending_connection: Option<(String, usize)>,
    next_id: usize,
    defs: Vec<ProcessorDef>,
    renderer_registry: NodeRendererRegistry,
}

impl NodeEditor {
    pub fn new(defs: Vec<ProcessorDef>) -> Self {
        let mut registry = NodeRendererRegistry::new(Arc::new(DefaultNodeRenderer));
        registry.register("TextInput", Arc::new(TextInputRenderer));

        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
            drag_start: Pos2::ZERO,
            drag_node_start: Pos2::ZERO,
            pending_connection: None,
            next_id: 1,
            defs,
            renderer_registry: registry,
        }
    }

    pub fn add_node(&mut self, type_name: &str, pos: Pos2) {
        let Some(def) = self.defs.iter().find(|p| p.type_name == type_name) else { return };
        let id = format!("node_{}", self.next_id);
        self.next_id += 1;

        self.nodes.push(Node {
            id,
            type_name: type_name.to_string(),
            label: def.label.clone(),
            kind: def.kind.clone(),
            pos,
            inputs: def.inputs.iter().map(|_| NodeInput::default()).collect(),
        });
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui) {
        let canvas_rect = ui.available_rect_before_wrap();
        let painter = ui.painter_at(canvas_rect);

        self.draw_grid(&painter, canvas_rect);

        let _canvas_response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());

        // Handle panning with secondary or middle mouse button
        let is_panning = ctx.input(|i| {
            let mouse_over_canvas = canvas_rect.contains(i.pointer.hover_pos().unwrap_or(Pos2::ZERO));
            mouse_over_canvas && (i.pointer.button_down(PointerButton::Secondary) || i.pointer.button_down(PointerButton::Middle))
        });
        if is_panning {
            let delta = ctx.input(|i| i.pointer.delta());
            self.pan_offset += delta;
        }

        // Handle zooming with scroll wheel
        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        if canvas_rect.contains(ctx.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO)) && scroll_delta != 0.0 {
            let zoom_factor = 1.1_f32;
            let old_zoom = self.zoom;
            self.zoom = (self.zoom * zoom_factor.powf(scroll_delta * 0.01)).clamp(0.3, 3.0);

            if let Some(mouse_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                let zoom_change = self.zoom / old_zoom;
                let delta = mouse_pos - canvas_rect.min;
                self.pan_offset = (self.pan_offset - delta) * zoom_change + delta;
            }
        }

        if ctx.input(|i| i.pointer.primary_clicked()) {
            self.pending_connection = None;
        }

        // Draw existing connections
        for conn in &self.connections {
            if let (Some(from_node), Some(to_node)) = (
                self.nodes.iter().find(|n| n.id == conn.from_node),
                self.nodes.iter().find(|n| n.id == conn.to_node),
            ) {
                let from_pos = output_slot_pos(from_node, self.pan_offset, canvas_rect.min, self.zoom);
                let to_pos = input_slot_pos(to_node, conn.to_slot, self.pan_offset, canvas_rect.min, self.zoom);
                draw_bezier(&painter, from_pos, to_pos, Color32::from_rgb(100, 160, 255), 2.0);
            }
        }

        // Pending connection preview
        if let Some((ref src_id, _)) = self.pending_connection.clone() {
            if let Some(src) = self.nodes.iter().find(|n| n.id == *src_id) {
                let from_pos = output_slot_pos(src, self.pan_offset, canvas_rect.min, self.zoom);
                if let Some(mouse) = ctx.input(|i| i.pointer.hover_pos()) {
                    draw_bezier(&painter, from_pos, mouse, Color32::from_rgb(100, 160, 255).gamma_multiply(0.5), 1.5);
                }
            }
        }

        // Draw nodes using their respective renderers
        let node_ids: Vec<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        for node_id in node_ids {
            self.show_node(ui, &painter, &node_id, ctx, canvas_rect);
        }

        if ctx.input(|i| i.pointer.primary_released()) {
            self.dragging_node = None;
        }
    }

    fn show_node(&mut self, ui: &mut Ui, painter: &Painter, node_id: &str, ctx: &Context, canvas_rect: Rect) {
        let idx = self.nodes.iter().position(|n| n.id == node_id).unwrap();
        let node_pos = (self.nodes[idx].pos * self.zoom) + self.pan_offset + canvas_rect.min.to_vec2();

        let node = &self.nodes[idx];
        let def = self.defs.iter()
            .find(|d| d.type_name == node.type_name)
            .cloned()
            .unwrap_or_else(|| ProcessorDef {
                type_name: "Unknown".to_string(),
                label: "Unknown".to_string(),
                description: String::new(),
                color: [100, 100, 130],
                kind: NodeKind::Pipeline,
                inputs: vec![],
                output: None,
            });

        // Get the appropriate renderer for this node type
        let renderer = self.renderer_registry.get(&node.type_name);

        // Render the node
        renderer.render(node, &def, painter, ui, ctx, node_pos, self.zoom, self.pan_offset, canvas_rect);

        // Handle node dragging (for all node types)
        self.handle_node_drag(ui, ctx, idx, node_pos);

        // Handle node interactions (slots, connections)
        self.handle_node_interactions(ui, node_id, idx, node_pos);
    }

    fn handle_node_drag(&mut self, ui: &mut Ui, _ctx: &Context, idx: usize, node_pos: Pos2) {
        let scaled_header = 32.0 * self.zoom; // HEADER_HEIGHT
        let scaled_width = 180.0 * self.zoom; // NODE_WIDTH (default)
        let header_rect = Rect::from_min_size(node_pos, Vec2::new(scaled_width, scaled_header));

        let header_resp = ui.allocate_rect(header_rect, Sense::click_and_drag());
        if header_resp.drag_started() {
            self.dragging_node = Some(self.nodes[idx].id.clone());
            self.drag_start = header_resp.interact_pointer_pos().unwrap_or_default();
            self.drag_node_start = self.nodes[idx].pos;
        }
        if self.dragging_node.as_deref() == Some(&self.nodes[idx].id) {
            if let Some(ptr) = header_resp.interact_pointer_pos() {
                self.nodes[idx].pos = self.drag_node_start + (ptr - self.drag_start) / self.zoom;
            }
        }
    }

    fn handle_node_interactions(&mut self, ui: &mut Ui, node_id: &str, idx: usize, node_pos: Pos2) {
        let zoom = self.zoom;

        // Input slots
        for slot_idx in 0..self.nodes[idx].inputs.len() {
            let scaled_slot = 22.0 * zoom; // SLOT_HEIGHT
            let scaled_header = 32.0 * zoom; // HEADER_HEIGHT
            let slot_y = node_pos.y + scaled_header + slot_idx as f32 * scaled_slot + scaled_slot / 2.0;
            let slot_center = Pos2::new(node_pos.x, slot_y);

            let dot_resp = ui.allocate_rect(Rect::from_center_size(slot_center, Vec2::splat(14.0 * zoom)), Sense::click());
            if dot_resp.clicked() {
                if let Some((src_id, src_slot)) = self.pending_connection.take() {
                    if src_id != node_id {
                        self.connections.retain(|c| !(c.to_node == node_id && c.to_slot == slot_idx));
                        self.nodes[idx].inputs[slot_idx].source_node = Some(src_id.clone());
                        self.nodes[idx].inputs[slot_idx].source_slot = Some(src_slot);
                        self.connections.push(Connection {
                            from_node: src_id,
                            from_slot: src_slot,
                            to_node: node_id.to_string(),
                            to_slot: slot_idx,
                        });
                    }
                }
            }
        }

        // Output slot
        let scaled_slot = 22.0 * zoom;
        let scaled_header = 32.0 * zoom;
        let scaled_width = 180.0 * zoom;
        let num_inputs = self.nodes[idx].inputs.len();
        let out_y = node_pos.y + scaled_header + num_inputs as f32 * scaled_slot / 2.0 + scaled_slot / 2.0;
        let out_center = Pos2::new(node_pos.x + scaled_width, out_y);

        let out_resp = ui.allocate_rect(Rect::from_center_size(out_center, Vec2::splat(14.0 * zoom)), Sense::click());
        if out_resp.clicked() {
            self.pending_connection = Some((node_id.to_string(), 0));
        }
    }

    fn draw_grid(&self, painter: &Painter, rect: Rect) {
        painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(12, 12, 20));
        let grid_size = 24.0 * self.zoom;
        let ox = self.pan_offset.x % grid_size;
        let oy = self.pan_offset.y % grid_size;
        let mut x = rect.min.x + ox;
        while x < rect.max.x {
            painter.line_segment([Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)], Stroke::new(0.5, Color32::from_rgb(25, 25, 38)));
            x += grid_size;
        }
        let mut y = rect.min.y + oy;
        while y < rect.max.y {
            painter.line_segment([Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)], Stroke::new(0.5, Color32::from_rgb(25, 25, 38)));
            y += grid_size;
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let nodes = self.nodes.iter()
            .filter(|n| matches!(n.kind, NodeKind::Pipeline))
            .map(|node| {
                let inputs = node.inputs.iter().map(|inp| {
                    if let Some(ref src) = inp.source_node {
                        InputJson { value: None, source_node: Some(src.clone()), source_slot: inp.source_slot }
                    } else {
                        InputJson { value: Some(inp.literal_value.clone()), source_node: None, source_slot: None }
                    }
                }).collect();

                NodeJson {
                    id: node.id.clone(),
                    type_name: node.type_name.clone(),
                    parameters: serde_json::Value::Object(Default::default()),
                    inputs,
                }
            })
            .collect();

        serde_json::to_string_pretty(&PipelineJson { nodes })
    }
}

fn output_slot_pos(node: &Node, pan: Vec2, canvas_min: Pos2, zoom: f32) -> Pos2 {
    let scaled_header = 32.0 * zoom;
    let scaled_slot = 22.0 * zoom;
    let scaled_width = 180.0 * zoom;
    let out_y = node.pos.y * zoom + scaled_header + node.inputs.len() as f32 * scaled_slot / 2.0 + scaled_slot / 2.0;
    Pos2::new(node.pos.x * zoom + scaled_width, out_y) + pan + canvas_min.to_vec2()
}

fn input_slot_pos(node: &Node, slot: usize, pan: Vec2, canvas_min: Pos2, zoom: f32) -> Pos2 {
    let scaled_header = 32.0 * zoom;
    let scaled_slot = 22.0 * zoom;
    let slot_y = node.pos.y * zoom + scaled_header + slot as f32 * scaled_slot + scaled_slot / 2.0;
    Pos2::new(node.pos.x * zoom, slot_y) + pan + canvas_min.to_vec2()
}

fn draw_bezier(painter: &Painter, from: Pos2, to: Pos2, color: Color32, width: f32) {
    let dx = (to.x - from.x).abs().max(80.0);
    let cp1 = Pos2::new(from.x + dx * 0.5, from.y);
    let cp2 = Pos2::new(to.x - dx * 0.5, to.y);
    let points: Vec<Pos2> = (0..=32).map(|i| {
        let t = i as f32 / 32.0;
        let u = 1.0 - t;
        Pos2::new(
            u*u*u*from.x + 3.0*u*u*t*cp1.x + 3.0*u*t*t*cp2.x + t*t*t*to.x,
            u*u*u*from.y + 3.0*u*u*t*cp1.y + 3.0*u*t*t*cp2.y + t*t*t*to.y,
        )
    }).collect();
    for w in points.windows(2) {
        painter.line_segment([w[0], w[1]], Stroke::new(width, color));
    }
}
