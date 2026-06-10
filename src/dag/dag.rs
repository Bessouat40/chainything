use crate::dag::node::Node;

struct Dag {
    nodes: Vec<Node>,
}

impl Dag {
    fn new() -> Self {
        Dag { nodes: Vec::new() }
    }

    fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    fn execute(&self) -> Result<(), String> {
        for node in &self.nodes {
            if let Some(ref input_node) = node.input {
                if let Some(ref output) = input_node.output {
                    node.execute(output.as_ref())
                        .map_err(|e| format!("Error executing node {}: {:?}", node.id, e))?;
                } else {
                    return Err(format!("Input for node {} is not ready.", node.id));
                }
            } else {
                return Err(format!("Node {} has no input.", node.id));
            }
        }
        Ok(())
    }
}