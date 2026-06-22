use egui::{Painter, Rect, Pos2, Ui, Context};
use crate::types::{Node, ProcessorDef};

/// Core trait for rendering a node type
/// Implement this to customize how a node looks and behaves
pub trait NodeRenderer: Send + Sync {
    /// Draw the node on the canvas
    fn render(
        &self,
        node: &Node,
        def: &ProcessorDef,
        painter: &Painter,
        ui: &mut Ui,
        ctx: &Context,
        node_pos: Pos2,
        zoom: f32,
        pan_offset: egui::Vec2,
        canvas_rect: Rect,
    );

    /// Called when rendering is complete to handle interaction state
    fn on_render_complete(
        &self,
        _node: &Node,
        _def: &ProcessorDef,
    ) {
        // Override if you need post-render cleanup
    }
}
