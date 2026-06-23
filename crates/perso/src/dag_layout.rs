use egui::*;
use crate::nodes::node::Node;

pub struct DAGLayout {
    nodes: Vec<Node>,
}

impl Default for DAGLayout {

    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl DAGLayout {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        nodes.push(Node::new(pos2(300.0, 300.0)));
        nodes.push(Node::new(pos2(700.0, 700.0)));
        Self { nodes }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show_inside(ui, |ui| {

                for node in &mut self.nodes {
                    node.show(ui);
                }
            });
    }

    pub fn add_node(&mut self, pos: Pos2) {
        let new_node = Node::new(pos);
        self.nodes.push(new_node);
    }
}