#![allow(clippy::use_self)]

use crate::nodes::base_node::BaseNode;
use crate::nodes::node_registry::NodeRegistry;

use std::mem::discriminant;

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, SnarlViewer},
};

pub struct DemoViewer {
    pub node_registry: NodeRegistry,
}

impl DemoViewer {
    pub fn new() -> Self {
        Self {
            node_registry: NodeRegistry::new(),
        }
    }
}

impl SnarlViewer<Box<dyn BaseNode>> for DemoViewer {
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Box<dyn BaseNode>>) {
        if !to.remotes.is_empty() {
            return;
        }
        let out_pin_idx = from.id.output;
        let in_pin_idx = to.id.input;

        let from_node = &snarl[from.id.node];
        let to_node = &snarl[to.id.node];

        if let (Some(out_map), Some(in_map)) = (from_node.mapping_output(), to_node.mapping_input())
            && let (Some(out_type), Some(in_type)) =
                (out_map.get(&out_pin_idx), in_map.get(&in_pin_idx))
            && discriminant(out_type) == discriminant(in_type)
        {
            snarl.connect(from.id, to.id);
        }
    }

    fn title(&mut self, node: &Box<dyn BaseNode>) -> String {
        node.name().to_string()
    }

    fn inputs(&mut self, node: &Box<dyn BaseNode>) -> usize {
        node.inputs_count()
    }

    fn outputs(&mut self, node: &Box<dyn BaseNode>) -> usize {
        node.outputs_count()
    }

    #[allow(refining_impl_trait)]
    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
    ) -> PinInfo {
        snarl[pin.id.node].show_input(pin, ui)
    }

    #[allow(refining_impl_trait)]
    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
    ) -> PinInfo {
        snarl[pin.id.node].show_output(pin, ui)
    }

    #[inline]
    fn has_body(&mut self, node: &Box<dyn BaseNode>) -> bool {
        node.has_body()
    }

    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
    ) {
        let snarl_ref = &*snarl;
        let base_node = &snarl_ref[node];

        base_node.show_body(node, inputs, outputs, ui, snarl_ref);
    }

    fn has_node_menu(&mut self, _node: &Box<dyn BaseNode>) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
    ) {
        ui.label("Node menu");
        if ui.button("Duplicate").clicked() {
            let copy = snarl[node].clone();
            let pos = snarl
                .get_node_info(node)
                .map(|info| info.pos + egui::vec2(30.0, 30.0))
                .unwrap_or(egui::Pos2::ZERO);
            snarl.insert_node(pos, copy);
            ui.close();
        }
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }

    fn header_frame(
        &mut self,
        frame: egui::Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<Box<dyn BaseNode>>,
    ) -> egui::Frame {
        let get_node = &snarl[node];
        get_node.header_frame(frame)
    }
}
