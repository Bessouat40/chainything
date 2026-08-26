use std::sync::Arc;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::pipeline_core::pipeline::{InputSource, Pipeline, PipelineErrors};
use crate::pipeline_core::registry::ProcessorRegistry;
use crate::processors::iter::ForEachProcessor;

/// Defines the types of inputs possible for a pipeline node.
///
/// Used for deserializing the JSON configuration. This is marked as `#[serde(untagged)]`,
/// meaning the deserializer will automatically determine the variant based on the JSON structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum JsonInputDef {
    /// A connection sourced from another node in the pipeline.
    Connection {
        source_node: String,
        source_slot: usize,
    },
    /// A static value provided directly within the configuration.
    Static { value: String },
}

/// Represents the structure of a node in the JSON configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonNodeDef {
    /// Unique identifier for the node.
    pub id: String,
    /// The type of processor to instantiate (e.g., "filter", "transform").
    #[serde(rename = "type")]
    pub node_type: String,
    /// List of inputs required by this node.
    pub inputs: Vec<JsonInputDef>,
    /// Optional parameters for the processor (e.g., radius, threshold).
    /// Uses IndexMap to preserve parameter order from JSON.
    #[serde(default)]
    pub params: Option<IndexMap<String, serde_json::Value>>,
    /// Nested sub-pipeline definition, used by higher-order nodes such as `ForEach`.
    ///
    /// When present, this node applies the sub-pipeline to each element of its input
    /// collection instead of being built as a plain registry processor. The `ForEach`
    /// node reads `input_node`, `output_node`, and `output_slot` from [`Self::params`].
    #[serde(default)]
    pub sub_pipeline: Option<JsonPipelineDef>,
}

/// Represents the root structure of a pipeline definition in JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonPipelineDef {
    /// An ordered list of nodes that compose the pipeline.
    pub nodes: Vec<JsonNodeDef>,
}

/// Builder responsible for transforming a JSON definition into an executable `Pipeline` object.
pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Builds a `Pipeline` from a JSON string and a processor registry.
    ///
    /// # Arguments
    ///
    /// * `json_str` - A string slice containing the pipeline JSON definition.
    /// * `registry` - An instance of `ProcessorRegistry` used to instantiate processors by type.
    ///
    /// # Errors
    ///
    /// Returns `PipelineErrors::ComputingError` if:
    /// - The JSON is malformed or does not match the expected structure.
    /// - The `ProcessorRegistry` fails to create a processor for a given type.
    pub fn build_from_json(
        json_str: &str,
        registry: &Arc<ProcessorRegistry>,
    ) -> Result<Pipeline, PipelineErrors> {
        let def: JsonPipelineDef = serde_json::from_str(json_str)
            .map_err(|e| PipelineErrors::ComputingError(format!("Failed to parse JSON: {}", e)))?;

        Self::build_pipeline(&def, registry)
    }

    /// Builds a `Pipeline` from an already-parsed [`JsonPipelineDef`].
    ///
    /// This is the reusable core shared by [`Self::build_from_json`] and by
    /// [`ForEachProcessor`](crate::processors::iter::ForEachProcessor), which rebuilds
    /// its sub-pipeline from a nested definition once per element. Sharing this path
    /// means nested `ForEach` nodes compose without any special handling.
    ///
    /// The `registry` is taken as an `Arc` so it can be cheaply cloned into each
    /// higher-order node that needs to rebuild sub-pipelines at execution time.
    pub fn build_pipeline(
        def: &JsonPipelineDef,
        registry: &Arc<ProcessorRegistry>,
    ) -> Result<Pipeline, PipelineErrors> {
        let mut pipeline = Pipeline::new();

        for node_def in &def.nodes {
            let inputs = Self::resolve_inputs(&node_def.inputs);

            if node_def.node_type == "ForEach" {
                let processor = Self::build_foreach(node_def, registry)?;
                // No param->static expansion here: ForEach reads its config from
                // `params` (input_node/output_node/output_slot), which must not be
                // fed to the processor as data inputs.
                pipeline.add_processor(Box::new(processor), inputs);
                continue;
            }

            let processor = registry
                .build_processor(&node_def.node_type, node_def.id.clone())
                .map_err(|e| PipelineErrors::ComputingError(format!("Registry error: {}", e)))?;

            let mut inputs = inputs;
            Self::append_params_as_static(&mut inputs, node_def.params.as_ref());
            pipeline.add_processor(processor, inputs);
        }

        Ok(pipeline)
    }

    /// Converts JSON input definitions into pipeline [`InputSource`]s.
    fn resolve_inputs(json_inputs: &[JsonInputDef]) -> Vec<InputSource> {
        let mut inputs = Vec::with_capacity(json_inputs.len());
        for json_input in json_inputs {
            match json_input {
                JsonInputDef::Connection {
                    source_node,
                    source_slot,
                } => {
                    inputs.push(InputSource::connection(source_node.clone(), *source_slot));
                }
                JsonInputDef::Static { value } => {
                    inputs.push(InputSource::Static(Arc::new(value.clone())));
                }
            }
        }
        inputs
    }

    /// Appends optional `params` to a node's inputs as trailing static values,
    /// preserving the existing string/number coercion behaviour.
    fn append_params_as_static(
        inputs: &mut Vec<InputSource>,
        params: Option<&IndexMap<String, serde_json::Value>>,
    ) {
        let Some(params) = params else { return };
        for value in params.values() {
            match value {
                serde_json::Value::String(s) => {
                    if let Ok(u) = s.parse::<u32>() {
                        inputs.push(InputSource::Static(Arc::new(u)));
                    } else if let Ok(u) = s.parse::<u8>() {
                        inputs.push(InputSource::Static(Arc::new(u)));
                    } else {
                        inputs.push(InputSource::Static(Arc::new(s.clone())));
                    }
                }
                serde_json::Value::Number(n) => {
                    if let Some(u) = n.as_u64() {
                        inputs.push(InputSource::Static(Arc::new(u as u32)));
                    }
                }
                _ => {}
            }
        }
    }

    /// Constructs a [`ForEachProcessor`] from a node definition that carries a nested
    /// `sub_pipeline` and `input_node`/`output_node`/`output_slot` params.
    fn build_foreach(
        node_def: &JsonNodeDef,
        registry: &Arc<ProcessorRegistry>,
    ) -> Result<ForEachProcessor, PipelineErrors> {
        let sub_def = node_def.sub_pipeline.clone().ok_or_else(|| {
            PipelineErrors::ComputingError(format!(
                "ForEach node '{}' requires a 'sub_pipeline' definition",
                node_def.id
            ))
        })?;

        let params = node_def.params.as_ref();
        let input_node = Self::required_str_param(params, "input_node", &node_def.id)?;
        let output_node = Self::required_str_param(params, "output_node", &node_def.id)?;
        let output_slot = params
            .and_then(|p| p.get("output_slot"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        Ok(ForEachProcessor::new(
            node_def.id.clone(),
            Arc::clone(registry),
            sub_def,
            input_node,
            output_node,
            output_slot,
        ))
    }

    /// Reads a required string param, erroring with a clear message when absent.
    fn required_str_param(
        params: Option<&IndexMap<String, serde_json::Value>>,
        key: &str,
        node_id: &str,
    ) -> Result<String, PipelineErrors> {
        params
            .and_then(|p| p.get(key))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                PipelineErrors::ComputingError(format!(
                    "ForEach node '{}' requires a string param '{}'",
                    node_id, key
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline_core::builder::PipelineBuilder;
    use crate::pipeline_core::registry::ProcessorRegistry;

    #[test]
    fn test_build_pipeline_from_json() {
        let registry = std::sync::Arc::new(ProcessorRegistry::default());

        let json_data = r#"{
            "nodes": [
                {
                    "id": "reader",
                    "type": "ImageReader",
                    "inputs": [{"value": "./chat.jpg"}]
                },
                {
                    "id": "greyscale",
                    "type": "Greyscale",
                    "inputs": [{"source_node": "reader", "source_slot": 0}]
                },
                {
                    "id": "saver",
                    "type": "ImageSave",
                    "inputs": [
                        {"source_node": "greyscale", "source_slot": 0},
                        {"value": "./output.png"}
                    ]
                }
            ]
        }"#;

        let result = PipelineBuilder::build_from_json(json_data, &registry);

        assert!(result.is_ok());
    }

    #[test]
    fn test_build_foreach_from_nested_json() {
        let registry = std::sync::Arc::new(ProcessorRegistry::default());

        // A ForEach node carrying an inline sub-pipeline. We only assert that the
        // builder wires it up without error; execution semantics are covered by the
        // ForEachProcessor unit tests.
        let json_data = r#"{
            "nodes": [
                {
                    "id": "loop",
                    "type": "ForEach",
                    "inputs": [{"source_node": "lister", "source_slot": 0}],
                    "params": {"input_node": "item", "output_node": "grey", "output_slot": 0},
                    "sub_pipeline": {
                        "nodes": [
                            {"id": "item", "type": "ItemInput", "inputs": []},
                            {"id": "grey", "type": "Greyscale", "inputs": [{"source_node": "item", "source_slot": 0}]}
                        ]
                    }
                }
            ]
        }"#;

        let result = PipelineBuilder::build_from_json(json_data, &registry);
        assert!(result.is_ok(), "ForEach build failed: {:?}", result.err());
    }

    #[test]
    fn test_foreach_missing_params_errors() {
        let registry = std::sync::Arc::new(ProcessorRegistry::default());

        let json_data = r#"{
            "nodes": [
                {
                    "id": "loop",
                    "type": "ForEach",
                    "inputs": [],
                    "sub_pipeline": {"nodes": []}
                }
            ]
        }"#;

        // Missing input_node/output_node params must surface a clear build error.
        assert!(PipelineBuilder::build_from_json(json_data, &registry).is_err());
    }
}
