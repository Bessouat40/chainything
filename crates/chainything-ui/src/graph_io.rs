use egui::Pos2;
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use serde::{Deserialize, Serialize};

use crate::nodes::{base_node::BaseNode, node_registry::NodeRegistry};

/// Current on-disk schema version. Bump when the format changes incompatibly.
const GRAPH_FILE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphFile {
    pub version: u32,
    pub nodes: Vec<NodeEntry>,
    pub connections: Vec<ConnectionEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeEntry {
    /// Registry key used to recreate the node (matches [`BaseNode::name`]).
    #[serde(rename = "type")]
    pub node_type: String,
    /// Top-left position of the node on the canvas, `[x, y]`.
    pub pos: [f32; 2],
    /// Whether the node is expanded (not collapsed) on the canvas.
    #[serde(default = "default_open")]
    pub open: bool,
    /// Editable parameters, indexed as in [`BaseNode::get_parameter`].
    #[serde(default)]
    pub params: Vec<String>,
}

fn default_open() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionEntry {
    /// Index (into [`GraphFile::nodes`]) of the source node.
    pub from_node: usize,
    pub from_output: usize,
    /// Index (into [`GraphFile::nodes`]) of the destination node.
    pub to_node: usize,
    pub to_input: usize,
}

/// Reads every editable parameter of a node, stopping at the first gap.
fn collect_params(node: &dyn BaseNode) -> Vec<String> {
    let mut params = Vec::new();
    let mut idx = 0;
    while let Some(value) = node.get_parameter(idx) {
        params.push(value);
        idx += 1;
    }
    params
}

/// Serializes the whole editor graph to a pretty-printed JSON string.
pub fn serialize_graph(snarl: &Snarl<Box<dyn BaseNode>>) -> Result<String, String> {
    // Capture node ids in a stable order and map each to its array index so
    // wires can reference nodes positionally.
    let mut order: Vec<NodeId> = Vec::new();
    let mut index_of: std::collections::HashMap<NodeId, usize> = std::collections::HashMap::new();
    let mut nodes = Vec::new();

    for (node_id, node) in snarl.node_ids() {
        index_of.insert(node_id, order.len());
        order.push(node_id);

        let info = snarl.get_node_info(node_id);
        let (pos, open) = info
            .map(|i| ([i.pos.x, i.pos.y], i.open))
            .unwrap_or(([0.0, 0.0], true));

        nodes.push(NodeEntry {
            node_type: node.name().to_string(),
            pos,
            open,
            params: collect_params(node.as_ref()),
        });
    }

    let mut connections = Vec::new();
    for (out_pin, in_pin) in snarl.wires() {
        let (Some(&from_node), Some(&to_node)) =
            (index_of.get(&out_pin.node), index_of.get(&in_pin.node))
        else {
            continue;
        };
        connections.push(ConnectionEntry {
            from_node,
            from_output: out_pin.output,
            to_node,
            to_input: in_pin.input,
        });
    }

    let file = GraphFile {
        version: GRAPH_FILE_VERSION,
        nodes,
        connections,
    };
    serde_json::to_string_pretty(&file).map_err(|e| format!("serialization error: {e}"))
}

/// Rebuilds the editor graph from a JSON string, replacing the current contents.
///
/// Unknown node types are skipped (their wires are dropped too) so a graph saved
/// by a newer build still loads what it can rather than failing outright.
pub fn deserialize_graph(
    json: &str,
    registry: &NodeRegistry,
    snarl: &mut Snarl<Box<dyn BaseNode>>,
) -> Result<(), String> {
    let file: GraphFile =
        serde_json::from_str(json).map_err(|e| format!("invalid graph JSON: {e}"))?;

    if file.version > GRAPH_FILE_VERSION {
        return Err(format!(
            "graph file version {} is newer than supported version {}",
            file.version, GRAPH_FILE_VERSION
        ));
    }

    // Build into a fresh graph first so a malformed entry can't leave the editor
    // half-cleared.
    let mut new_snarl: Snarl<Box<dyn BaseNode>> = Snarl::new();
    // Maps each node's array index to the NodeId it got in the new graph; `None`
    // for nodes that could not be recreated (unknown type).
    let mut ids: Vec<Option<NodeId>> = Vec::with_capacity(file.nodes.len());

    for entry in &file.nodes {
        match registry.create_node(&entry.node_type) {
            Some(mut node) => {
                for (idx, value) in entry.params.iter().enumerate() {
                    node.set_parameter(idx, value);
                }
                let id = new_snarl.insert_node(Pos2::new(entry.pos[0], entry.pos[1]), node);
                if !entry.open
                    && let Some(info) = new_snarl.get_node_info_mut(id)
                {
                    info.open = false;
                }
                ids.push(Some(id));
            }
            None => {
                eprintln!("⚠ Unknown node type '{}', skipping", entry.node_type);
                ids.push(None);
            }
        }
    }

    for conn in &file.connections {
        let (Some(Some(&from)), Some(Some(&to))) = (
            ids.get(conn.from_node).map(Option::as_ref),
            ids.get(conn.to_node).map(Option::as_ref),
        ) else {
            continue;
        };
        new_snarl.connect(
            OutPinId {
                node: from,
                output: conn.from_output,
            },
            InPinId {
                node: to,
                input: conn.to_input,
            },
        );
    }

    *snarl = new_snarl;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_nodes_params_and_connections() {
        let registry = NodeRegistry::new();
        let mut snarl: Snarl<Box<dyn BaseNode>> = Snarl::new();

        let mut input = registry.create_node("TextInputNode").unwrap();
        input.set_parameter(0, "hello");
        let input_id = snarl.insert_node(Pos2::new(10.0, 20.0), input);

        let saver = registry.create_node("TextSave").unwrap();
        let saver_id = snarl.insert_node(Pos2::new(300.0, 40.0), saver);

        snarl.connect(
            OutPinId {
                node: input_id,
                output: 0,
            },
            InPinId {
                node: saver_id,
                input: 0,
            },
        );

        let json = serialize_graph(&snarl).expect("serialize");

        let mut restored: Snarl<Box<dyn BaseNode>> = Snarl::new();
        deserialize_graph(&json, &registry, &mut restored).expect("deserialize");

        assert_eq!(restored.node_ids().count(), 2);
        assert_eq!(restored.wires().count(), 1);

        let text_input = restored
            .node_ids()
            .map(|(_, n)| n)
            .find(|n| n.name() == "TextInputNode")
            .expect("text input restored");
        assert_eq!(text_input.get_parameter(0).as_deref(), Some("hello"));
    }
}
