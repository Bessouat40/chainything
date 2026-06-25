use std::collections::HashMap;

use crate::nodes::{
    base_node::BaseNode, image_display_node::ImageDisplayNode, image_reader_node::ImageReaderNode,
    text_input_node::TextInputNode,
};

pub struct NodeRegistry {
    pub nodes: HashMap<String, Box<dyn BaseNode>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let nodes = Self::create_node_registry();
        Self { nodes }
    }

    pub fn get_available_nodes(&self) -> Vec<&String> {
        self.nodes.keys().collect()
    }

    fn create_node_registry() -> HashMap<String, Box<dyn BaseNode>> {
        [
            (
                TextInputNode::new().name().to_string(),
                Box::new(TextInputNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ImageReaderNode::new().name().to_string(),
                Box::new(ImageReaderNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ImageDisplayNode::new().name().to_string(),
                Box::new(ImageDisplayNode::new()) as Box<dyn BaseNode>,
            ),
        ]
        .into_iter()
        .collect()
    }

    pub fn _get_node(&self, node_name: String) -> Option<&dyn BaseNode> {
        match self.nodes.get(&node_name) {
            Some(n) => Some(n.as_ref()),
            None => None,
        }
    }

    pub fn create_node(&self, node_name: &str) -> Option<Box<dyn BaseNode>> {
        self.nodes.get(node_name).cloned()
    }
}
