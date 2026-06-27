use crate::nodes::base_node::{DisplayData, InputOutputType};
use crate::nodes::viewer::DemoViewer;
use crate::nodes::{base_node::BaseNode, node_registry::NodeRegistry};
use crate::payload_parser::{GraphPayload, InputPayload, NodePayload};
use chainything::prelude::*;
use chainything::processors::greyscale_processor::RawImage;
use egui::Ui;
use egui_snarl::{InPinId, NodeId, Snarl, ui::SnarlWidget};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// Outputs of every processor after a run, keyed by pipeline node id.
type ExecOutput = HashMap<String, Vec<Arc<dyn Any + Send + Sync>>>;

/// Links a display node to the processor output that feeds it, so results can be
/// routed back to the right node once a run finishes.
struct DisplayBinding {
    display: NodeId,
    source_id: String,
    slot: usize,
}

pub struct DAGLayout {
    pub snarl: Snarl<Box<dyn BaseNode>>,
    viewer: DemoViewer,
    running: Arc<Mutex<bool>>,
    result: Arc<Mutex<Option<Result<ExecOutput, String>>>>,
    bindings: Vec<DisplayBinding>,
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
            running: Arc::new(Mutex::new(false)),
            result: Arc::new(Mutex::new(None)),
            bindings: Vec::new(),
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

    /// `true` while a pipeline run is in progress on the background thread.
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Builds and executes the pipeline on a background thread.
    ///
    /// Display-node connections are captured up front so the produced outputs
    /// can be routed back to them once the run completes (see
    /// [`poll_results`](Self::poll_results)).
    pub fn run(&mut self) {
        if self.is_running() {
            return;
        }

        // Free stale results (and their GPU textures) so the run shows fresh
        // output and unconnected display nodes don't pin memory indefinitely.
        for (_id, node) in self.snarl.node_ids() {
            node.clear_display();
        }

        let json = self.export_to_json();
        self.bindings = compute_display_bindings(&self.snarl);

        let running = Arc::clone(&self.running);
        let result = Arc::clone(&self.result);
        *running.lock().unwrap() = true;
        *result.lock().unwrap() = None;

        std::thread::spawn(move || {
            let outcome = run_pipeline_collect(&json);
            *result.lock().unwrap() = Some(outcome);
            *running.lock().unwrap() = false;
        });
    }

    /// Routes the outputs of a finished run into the connected display nodes.
    ///
    /// Call once per frame; it is a cheap no-op until a run finishes.
    pub fn poll_results(&mut self) {
        let Some(outcome) = self.result.lock().unwrap().take() else {
            return;
        };

        match outcome {
            Ok(outputs) => {
                for binding in &self.bindings {
                    let Some(slots) = outputs.get(&binding.source_id) else {
                        continue;
                    };
                    let Some(data) = slots.get(binding.slot) else {
                        continue;
                    };
                    let Some(node) = self.snarl.get_node(binding.display) else {
                        continue;
                    };

                    if let Some(text) = data.downcast_ref::<String>() {
                        node.set_display(DisplayData::Text(text.clone()));
                    } else if let Some(image) = data.downcast_ref::<RawImage>() {
                        node.set_display(DisplayData::Image(image.clone()));
                    }
                }
            }
            Err(err) => eprintln!("✗ Pipeline error: {}", err),
        }
    }
}

/// Builds the pipeline from JSON, executes it and returns every output.
fn run_pipeline_collect(json: &str) -> Result<ExecOutput, String> {
    let registry = ProcessorRegistry::with_standard_processors();
    let mut pipeline = PipelineBuilder::build_from_json(json, &registry)
        .map_err(|e| format!("build error: {:?}", e))?;
    pipeline
        .execute()
        .map_err(|e| format!("execution error: {:?}", e))?;
    Ok(pipeline.collect_outputs())
}

/// Captures, for each display node, the processor output that feeds it.
///
/// Pipeline ids match the indexing used by [`generate_payload`], so the two
/// stay consistent as long as the graph is not mutated between calls.
fn compute_display_bindings(snarl: &Snarl<Box<dyn BaseNode>>) -> Vec<DisplayBinding> {
    let mut id_map = HashMap::new();
    for (index, (node_id, _)) in snarl.node_ids().enumerate() {
        id_map.insert(node_id, index.to_string());
    }

    let mut bindings = Vec::new();
    for (node_id, node) in snarl.node_ids() {
        if node.is_processor() {
            continue;
        }

        for input_idx in 0..node.inputs_count() {
            let in_pin = snarl.in_pin(InPinId {
                node: node_id,
                input: input_idx,
            });

            let Some(remote) = in_pin.remotes.first() else {
                continue;
            };

            let is_processor_source = snarl
                .get_node(remote.node)
                .map(|n| n.is_processor())
                .unwrap_or(false);

            if is_processor_source && let Some(source_id) = id_map.get(&remote.node) {
                bindings.push(DisplayBinding {
                    display: node_id,
                    source_id: source_id.clone(),
                    slot: remote.output,
                });
            }
        }
    }

    bindings
}

use std::collections::HashMap;

fn get_node_parameter(node: &dyn BaseNode, input_idx: usize) -> Option<String> {
    node.get_parameter(input_idx)
}

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
            let in_pin_id = egui_snarl::InPinId {
                node: n_id,
                input: input_idx,
            };
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
                        .and_then(|v| {
                            if let InputOutputType::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null);

                    inputs_payload.push(InputPayload::Value { value });
                }
            } else {
                let value = get_node_parameter(node.as_ref(), input_idx)
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null);
                inputs_payload.push(InputPayload::Value { value });
            }
        }

        let mut params = None;
        if let Some(param_value) = node.get_parameter(0) {
            let mut params_map = HashMap::new();
            params_map.insert(
                "param_0".to_string(),
                serde_json::Value::String(param_value),
            );

            let mut idx = 1;
            while let Some(param_value) = node.get_parameter(idx) {
                params_map.insert(
                    format!("param_{}", idx),
                    serde_json::Value::String(param_value),
                );
                idx += 1;
            }

            if !params_map.is_empty() {
                params = Some(params_map);
            }
        }

        payload.nodes.push(NodePayload {
            id: current_node_str_id,
            node_type: node.name().replace("Node", ""),
            inputs: inputs_payload,
            params,
        });
    }
    payload
}
