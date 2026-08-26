use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct GraphPayload {
    pub nodes: Vec<NodePayload>,
}

#[derive(Serialize, Debug)]
pub struct NodePayload {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub inputs: Vec<InputPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, serde_json::Value>>,
    /// Nested loop body for a `ForEach` node. Absent for every other node, so the
    /// backend's `JsonNodeDef::sub_pipeline` stays `None` and nothing else changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_pipeline: Option<GraphPayload>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum InputPayload {
    Connection {
        source_node: String,
        source_slot: usize,
    },
    Value {
        value: serde_json::Value,
    },
}
