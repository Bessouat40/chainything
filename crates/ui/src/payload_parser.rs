use serde::Serialize;

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
