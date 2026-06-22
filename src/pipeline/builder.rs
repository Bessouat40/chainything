use serde::Deserialize;
use serde_json::Value;

use crate::pipeline::registry::ProcessorRegistry;
use crate::pipeline::pipeline::{InputSource, Pipeline, PipelineErrors};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum JsonInputDef {
    Connection {
        source_node: String,
        source_slot: usize,
    },
    Static {
        value: String,
    }
}

#[derive(Debug, Deserialize)]
pub struct JsonNodeDef {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub parameters: Value, 
    pub inputs: Vec<JsonInputDef>,
}

#[derive(Debug, Deserialize)]
pub struct JsonPipelineDef {
    pub nodes: Vec<JsonNodeDef>,
}

pub struct PipelineBuilder;

impl PipelineBuilder {
    pub fn build_from_json(
        json_str: &str, 
        registry: &ProcessorRegistry
    ) -> Result<Pipeline, PipelineErrors> {
        
        let def: JsonPipelineDef = serde_json::from_str(json_str)
            .map_err(|e| PipelineErrors::ComputingError(format!("Failed to parse JSON: {}", e)))?;

        let mut pipeline = Pipeline::new();

        for node_def in def.nodes {
            
            let processor = registry.build_processor(
                &node_def.node_type, 
                node_def.id.clone(), 
                node_def.parameters
            ).map_err(|e| PipelineErrors::ComputingError(format!("Registry error: {}", e)))?;

            let mut inputs = Vec::new();
            for json_input in node_def.inputs {
                match json_input {
                    JsonInputDef::Connection { source_node, source_slot } => {
                        inputs.push(InputSource::connection(source_node, source_slot));
                    },
                    JsonInputDef::Static { value } => {
                        inputs.push(InputSource::Static(std::sync::Arc::new(value)));
                    }
                }
            }

            pipeline.add_processor(processor, inputs);
        }

        Ok(pipeline)
    }
}