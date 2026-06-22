use egui::{Color32, Pos2};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Node kind ────────────────────────────────────────────────────────────────

/// Whether a node is executed by chainything or handled by the UI itself.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Serialized into the JSON pipeline and executed by chainything
    Pipeline,
    /// Intercepted by the UI, never sent to chainything
    Ui,
}

impl Default for NodeKind {
    fn default() -> Self {
        NodeKind::Pipeline
    }
}

// ── Processor / node definitions ─────────────────────────────────────────────

/// Definition of a single input slot, as loaded from nodes.toml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SlotDef {
    pub label: String,
    #[serde(default)]
    pub accepts_literal: bool,
}

/// Definition of the output slot type
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OutputDef {
    pub type_name: String,
}

/// Full definition of a node type, as loaded from nodes.toml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProcessorDef {
    pub type_name: String,
    pub label: String,
    pub description: String,
    /// [R, G, B]
    pub color: [u8; 3],
    #[serde(default)]
    pub kind: NodeKind,
    #[serde(default)]
    pub inputs: Vec<SlotDef>,
    #[serde(default)]
    pub output: Option<OutputDef>,
}

impl ProcessorDef {
    pub fn color32(&self) -> Color32 {
        Color32::from_rgb(self.color[0], self.color[1], self.color[2])
    }
}

/// Top-level structure of nodes.toml
#[derive(Debug, Deserialize)]
struct NodesFile {
    #[serde(rename = "nodes")]
    nodes: Vec<ProcessorDef>,
}

/// Load node definitions from a TOML file.
/// Falls back to built-in defaults if the file is missing or invalid.
pub fn load_processor_defs(path: &Path) -> Vec<ProcessorDef> {
    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<NodesFile>(&content) {
            Ok(file) => {
                println!("[nodes] Loaded {} node(s) from {}", file.nodes.len(), path.display());
                file.nodes
            }
            Err(e) => {
                eprintln!("[nodes] Failed to parse {}: {e} — using built-in defaults", path.display());
                default_processor_defs()
            }
        },
        Err(_) => {
            eprintln!("[nodes] {} not found — using built-in defaults", path.display());
            default_processor_defs()
        }
    }
}

/// Built-in fallback definitions (mirrors nodes.toml defaults)
fn default_processor_defs() -> Vec<ProcessorDef> {
    vec![
        ProcessorDef {
            type_name: "ImageReader".to_string(),
            label: "Image Reader".to_string(),
            description: "Reads an image from disk".to_string(),
            color: [80, 160, 255],
            kind: NodeKind::Pipeline,
            inputs: vec![SlotDef { label: "Path".to_string(), accepts_literal: true }],
            output: Some(OutputDef { type_name: "Image".to_string() }),
        },
        ProcessorDef {
            type_name: "Greyscale".to_string(),
            label: "Greyscale".to_string(),
            description: "Converts image to greyscale".to_string(),
            color: [160, 100, 255],
            kind: NodeKind::Pipeline,
            inputs: vec![SlotDef { label: "Image".to_string(), accepts_literal: false }],
            output: Some(OutputDef { type_name: "Image".to_string() }),
        },
        ProcessorDef {
            type_name: "ImageSave".to_string(),
            label: "Image Save".to_string(),
            description: "Saves image to disk".to_string(),
            color: [80, 210, 140],
            kind: NodeKind::Pipeline,
            inputs: vec![
                SlotDef { label: "Image".to_string(), accepts_literal: false },
                SlotDef { label: "Path".to_string(), accepts_literal: true },
            ],
            output: Some(OutputDef { type_name: "Path".to_string() }),
        },
    ]
}

// ── Runtime node state ───────────────────────────────────────────────────────

/// A node instance on the canvas
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub type_name: String,
    pub label: String,
    pub kind: NodeKind,
    pub pos: Pos2,
    pub inputs: Vec<NodeInput>,
}

#[derive(Debug, Clone, Default)]
pub struct NodeInput {
    pub source_node: Option<String>,
    pub source_slot: Option<usize>,
    pub literal_value: String,
}

/// A connection between two nodes
#[derive(Debug, Clone)]
pub struct Connection {
    pub from_node: String,
    pub from_slot: usize,
    pub to_node: String,
    pub to_slot: usize,
}

// ── JSON serialization toward chainything ────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct PipelineJson {
    pub nodes: Vec<NodeJson>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeJson {
    pub id: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub parameters: serde_json::Value,
    pub inputs: Vec<InputJson>,
}

#[derive(Serialize, Deserialize)]
pub struct InputJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_slot: Option<usize>,
}