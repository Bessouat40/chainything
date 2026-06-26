use serde::Deserialize;

use crate::pipeline_core::pipeline::{InputSource, Pipeline, PipelineErrors};
use crate::pipeline_core::registry::ProcessorRegistry;

/// Defines the types of inputs possible for a pipeline node.
///
/// Used for deserializing the JSON configuration. This is marked as `#[serde(untagged)]`,
/// meaning the deserializer will automatically determine the variant based on the JSON structure.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct JsonNodeDef {
    /// Unique identifier for the node.
    pub id: String,
    /// The type of processor to instantiate (e.g., "filter", "transform").
    #[serde(rename = "type")]
    pub node_type: String,
    /// List of inputs required by this node.
    pub inputs: Vec<JsonInputDef>,
    /// Optional parameters for the processor (e.g., radius, threshold).
    #[serde(default)]
    pub params: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Represents the root structure of a pipeline definition in JSON.
#[derive(Debug, Deserialize)]
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
        registry: &ProcessorRegistry,
    ) -> Result<Pipeline, PipelineErrors> {
        let def: JsonPipelineDef = serde_json::from_str(json_str)
            .map_err(|e| PipelineErrors::ComputingError(format!("Failed to parse JSON: {}", e)))?;

        let mut pipeline = Pipeline::new();

        for node_def in def.nodes {
            let processor = registry
                .build_processor(&node_def.node_type, node_def.id.clone())
                .map_err(|e| PipelineErrors::ComputingError(format!("Registry error: {}", e)))?;

            let mut inputs = Vec::new();
            for json_input in node_def.inputs {
                match json_input {
                    JsonInputDef::Connection {
                        source_node,
                        source_slot,
                    } => {
                        inputs.push(InputSource::connection(source_node, source_slot));
                    }
                    JsonInputDef::Static { value } => {
                        inputs.push(InputSource::Static(std::sync::Arc::new(value)));
                    }
                }
            }

            if let Some(params) = node_def.params {
                for (_key, value) in params {
                    match value {
                        serde_json::Value::String(s) => {
                            if let Ok(u) = s.parse::<u32>() {
                                inputs.push(InputSource::Static(std::sync::Arc::new(u)));
                            } else if let Ok(u) = s.parse::<u8>() {
                                inputs.push(InputSource::Static(std::sync::Arc::new(u)));
                            } else {
                                inputs.push(InputSource::Static(std::sync::Arc::new(s)));
                            }
                        }
                        serde_json::Value::Number(n) => {
                            if let Some(u) = n.as_u64() {
                                inputs.push(InputSource::Static(std::sync::Arc::new(u as u32)));
                            }
                        }
                        _ => {}
                    }
                }
            }

            pipeline.add_processor(processor, inputs);
        }

        Ok(pipeline)
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline_core::builder::PipelineBuilder;
    use crate::pipeline_core::registry::ProcessorRegistry;

    #[test]
    fn test_build_pipeline_from_json() {
        let registry = ProcessorRegistry::default();

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
}
