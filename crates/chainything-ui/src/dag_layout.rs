use crate::nodes::base_node::{DisplayData, InputOutputType};
use crate::nodes::viewer::DemoViewer;
use crate::nodes::{base_node::BaseNode, node_registry::NodeRegistry};
use crate::payload_parser::{GraphPayload, InputPayload, NodePayload};
use chainything::prelude::*;
use chainything::processors::images::greyscale_processor::RawImage;
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

    /// Prompts for a destination file and writes the full editor graph to it as
    /// JSON (see [`crate::graph_io`]). A no-op if the user cancels the dialog.
    pub fn save_graph_to_file(&self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Chainything graph", &["json"])
            .set_file_name("graph.json")
            .save_file()
        else {
            return;
        };

        match crate::graph_io::serialize_graph(&self.snarl) {
            Ok(json) => {
                if let Err(err) = std::fs::write(&path, json) {
                    eprintln!("✗ Failed to write {}: {}", path.display(), err);
                }
            }
            Err(err) => eprintln!("✗ Export failed: {}", err),
        }
    }

    /// Prompts for a JSON file and replaces the current graph with its contents.
    /// A no-op if the user cancels the dialog; errors are logged and leave the
    /// existing graph untouched.
    pub fn load_graph_from_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Chainything graph", &["json"])
            .pick_file()
        else {
            return;
        };

        let json = match std::fs::read_to_string(&path) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("✗ Failed to read {}: {}", path.display(), err);
                return;
            }
        };

        if let Err(err) =
            crate::graph_io::deserialize_graph(&json, &self.viewer.node_registry, &mut self.snarl)
        {
            eprintln!("✗ Import failed: {}", err);
            return;
        }

        // The previous run's display bindings reference nodes that no longer
        // exist; drop them so stale results aren't routed into the new graph.
        self.bindings.clear();
    }

    /// Loads a graph from a JSON string.
    pub fn load_graph_from_json(&mut self, json: &str) -> Result<(), String> {
        crate::graph_io::deserialize_graph(json, &self.viewer.node_registry, &mut self.snarl)?;
        self.bindings.clear();
        Ok(())
    }

    /// Removes every node and connection, resetting the editor to an empty graph.
    pub fn clear_graph(&mut self) {
        self.snarl = Snarl::new();
        // The previous run's display bindings reference nodes that no longer
        // exist; drop them so stale results aren't routed into the empty graph.
        self.bindings.clear();
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

        for (_id, node) in self.snarl.node_ids() {
            node.clear_display();
        }

        route_direct_display_values(&self.snarl);

        self.bindings = compute_display_bindings(&self.snarl);

        let pipeline = match build_pipeline_from_snarl(&self.snarl) {
            Ok(pipeline) => pipeline,

            Err(err) => {
                eprintln!("✗ Pipeline build error: {}", err);
                return;
            }
        };

        let running = Arc::clone(&self.running);
        let result = Arc::clone(&self.result);

        *running.lock().unwrap() = true;
        *result.lock().unwrap() = None;

        std::thread::spawn(move || {
            let outcome = run_pipeline_collect(pipeline);

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

fn input_output_to_source(value: &InputOutputType) -> Result<InputSource, String> {
    match value {
        InputOutputType::String(value) => {
            Ok(InputSource::static_data(value.clone()))
        }

        InputOutputType::RawImage(Some(image)) => {
            Ok(InputSource::static_data(image.clone()))
        }

        InputOutputType::RawImage(None) => {
            Err("RawImage source contains no image".to_string())
        }

        InputOutputType::Mesh3D(Some(mesh)) => {
            Ok(InputSource::static_data(mesh.clone()))
        }

        InputOutputType::Mesh3D(None) => {
            Err("Mesh3D source contains no mesh".to_string())
        }

        InputOutputType::Llm => {
            Err("LLM values cannot be provided directly as static node values".to_string())
        }
    }
}

fn build_pipeline_from_snarl(
    snarl: &Snarl<Box<dyn BaseNode>>,
) -> Result<Pipeline, String> {
    let registry = ProcessorRegistry::with_standard_processors();
    let mut pipeline = Pipeline::new();

    let mut id_map = HashMap::new();

    for (index, (node_id, _)) in snarl.node_ids().enumerate() {
        id_map.insert(node_id, index.to_string());
    }

    for (node_id, node) in snarl.node_ids() {
        if !node.is_processor() {
            continue;
        }

        let processor_id = id_map
            .get(&node_id)
            .ok_or_else(|| {
                format!(
                    "Unable to find pipeline id for node {}",
                    node.name()
                )
            })?
            .clone();

        let processor_type = node.name().replace("Node", "");

        let processor = registry
            .build_processor(
                &processor_type,
                processor_id.clone(),
            )
            .map_err(|err| {
                format!(
                    "Failed to build processor {}: {}",
                    processor_type,
                    err
                )
            })?;

        let mut inputs = Vec::new();

        for input_idx in 0..node.inputs_count() {
            let in_pin_id = InPinId {
                node: node_id,
                input: input_idx,
            };

            let in_pin = snarl.in_pin(in_pin_id);

            let Some(out_pin) = in_pin.remotes.first() else {
                continue;
            };

            let source_node = snarl
                .get_node(out_pin.node)
                .ok_or_else(|| {
                    format!(
                        "Source node not found for input {} of {}",
                        input_idx,
                        node.name()
                    )
                })?;

            if source_node.is_processor() {
                let source_node_id = id_map
                    .get(&out_pin.node)
                    .ok_or_else(|| {
                        format!(
                            "Unable to find pipeline id for source processor {}",
                            source_node.name()
                        )
                    })?
                    .clone();

                inputs.push(
                    InputSource::connection(
                        source_node_id,
                        out_pin.output,
                    )
                );

                continue;
            }

            let values = source_node
                .get_value()
                .ok_or_else(|| {
                    format!(
                        "Source node {} has no runtime value",
                        source_node.name()
                    )
                })?;

            let value = values
                .get(out_pin.output)
                .ok_or_else(|| {
                    format!(
                        "Output slot {} does not exist on source node {}",
                        out_pin.output,
                        source_node.name()
                    )
                })?;

            let source = input_output_to_source(value)?;

            inputs.push(source);
        }

        let mut parameter_index = 0;

        while let Some(parameter) = node.get_parameter(parameter_index) {
            inputs.push(
                InputSource::static_data(parameter)
            );

            parameter_index += 1;
        }

        pipeline.add_processor(
            processor,
            inputs,
        );
    }

    Ok(pipeline)
}

/// Builds the pipeline from JSON, executes it and returns every output.
fn run_pipeline_collect(
    mut pipeline: Pipeline,
) -> Result<ExecOutput, String> {
    pipeline
        .execute()
        .map_err(|e| format!("execution error: {:?}", e))?;

    Ok(pipeline.collect_outputs())
}


fn route_direct_display_values(snarl: &Snarl<Box<dyn BaseNode>>) {
    for (display_id, display_node) in snarl.node_ids() {
        if display_node.is_processor() || display_node.inputs_count() == 0 {
            continue;
        }

        for input_idx in 0..display_node.inputs_count() {
            let in_pin = snarl.in_pin(InPinId {
                node: display_id,
                input: input_idx,
            });

            let Some(remote) = in_pin.remotes.first() else {
                continue;
            };

            let Some(source_node) = snarl.get_node(remote.node) else {
                continue;
            };

            if source_node.is_processor() {
                continue;
            }

            let Some(values) = source_node.get_value() else {
                continue;
            };

            let Some(value) = values.get(remote.output) else {
                continue;
            };

            match value {
                InputOutputType::RawImage(Some(image)) => {
                    display_node.set_display(
                        DisplayData::Image(image.clone())
                    );
                }

                InputOutputType::String(text) => {
                    display_node.set_display(
                        DisplayData::Text(text.clone())
                    );
                }

                _ => {}
            }
        }
    }
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
