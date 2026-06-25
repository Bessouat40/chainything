use std::collections::HashMap;

use crate::nodes::{
    base_node::BaseNode, greyscale_node::GreyScaleNode, image_display_node::ImageDisplayNode, image_reader_node::ImageReaderNode, text_input_node::TextInputNode
};

pub struct NodeRegistry {
    pub nodes: HashMap<String, Box<dyn BaseNode>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let nodes = Self::create_node_registry();
        Self { nodes }
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
            (
                GreyScaleNode::new().name().to_string(),
                Box::new(GreyScaleNode::new()) as Box<dyn BaseNode>,
            ),
        ]
        .into_iter()
        .collect()
    }

    pub fn create_node(&self, node_name: &str) -> Option<Box<dyn BaseNode>> {
        self.nodes.get(node_name).cloned()
    }
}
