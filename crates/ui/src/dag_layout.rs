use crate::nodes::base_node::InputOutputType;
use crate::nodes::{base_node::BaseNode, node_registry::NodeRegistry};
use crate::nodes::viewer::DemoViewer;
use crate::payload_parser::{GraphPayload, InputPayload, NodePayload};
use egui::Ui;
use egui_snarl::{Snarl, ui::SnarlWidget};

pub struct DAGLayout {
    pub snarl: Snarl<Box<dyn BaseNode>>,
    viewer: DemoViewer,
}

impl Default for DAGLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DAGLayout {
    pub fn new() -> Self {
        let snarl = Snarl::new();
        let demo_viewer = DemoViewer::new();

        Self {
            snarl,
            viewer: demo_viewer,
        }
    }

    pub fn get_snarl_and_registry(&mut self) -> (&mut Snarl<Box<dyn BaseNode>>, &NodeRegistry) {
        (&mut self.snarl, &self.viewer.node_registry)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        SnarlWidget::new().show(&mut self.snarl, &mut self.viewer, ui);
    }

    pub fn export_to_json(&self) -> String {
        let payload = generate_payload(&self.snarl);
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
    }
}

use std::collections::HashMap;

pub fn generate_payload(snarl: &Snarl<Box<dyn BaseNode>>) -> GraphPayload {
    let mut payload = GraphPayload { nodes: Vec::new() };
    let mut id_map = HashMap::new();
    for (index, tuple) in snarl.node_ids().enumerate() {
        let n_id = tuple.0;
        id_map.insert(n_id, index.to_string());
    }
    for tuple in snarl.node_ids() {
        let n_id = tuple.0;
        let node = tuple.1;

        if !node.is_processor() {
            continue;
        }

        let mut inputs_payload = Vec::new();
        let current_node_str_id = id_map.get(&n_id).unwrap().clone();

        for input_idx in 0..node.inputs_count() {
            let in_pin_id = egui_snarl::InPinId { node: n_id, input: input_idx };
            let in_pin = snarl.in_pin(in_pin_id);

            if let Some(out_pin) = in_pin.remotes.first() {
                let source_node = snarl.get_node(out_pin.node).unwrap();

                if source_node.is_processor() {
                    let source_node_str = id_map.get(&out_pin.node).unwrap().clone();
                    inputs_payload.push(InputPayload::Connection {
                        source_node: source_node_str,
                        source_slot: out_pin.output,
                    });
                } else {
                    let value = source_node
                        .get_value()
                        .and_then(|vals| vals.get(out_pin.output))
                        .and_then(|v| if let InputOutputType::String(s) = v { Some(s.clone()) } else { None })
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null);

                    inputs_payload.push(InputPayload::Value { value });
                }
            } else {
                inputs_payload.push(InputPayload::Value {
                    value: serde_json::Value::Null,
                });
            }
        }

        payload.nodes.push(NodePayload {
            id: current_node_str_id,
            node_type: node.name().replace("Node", ""),
            inputs: inputs_payload,
        });
    }
    payload
}
