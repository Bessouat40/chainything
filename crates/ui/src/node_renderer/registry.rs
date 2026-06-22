use std::collections::HashMap;
use std::sync::Arc;
use crate::node_renderer::base::NodeRenderer;

pub struct NodeRendererRegistry {
    renderers: HashMap<String, Arc<dyn NodeRenderer>>,
    default_renderer: Arc<dyn NodeRenderer>,
}

impl NodeRendererRegistry {
    pub fn new(default_renderer: Arc<dyn NodeRenderer>) -> Self {
        Self {
            renderers: HashMap::new(),
            default_renderer,
        }
    }

    /// Register a custom renderer for a node type
    pub fn register(&mut self, node_type: &str, renderer: Arc<dyn NodeRenderer>) {
        self.renderers.insert(node_type.to_string(), renderer);
    }

    /// Get renderer for a node type, falls back to default
    pub fn get(&self, node_type: &str) -> Arc<dyn NodeRenderer> {
        self.renderers
            .get(node_type)
            .cloned()
            .unwrap_or_else(|| self.default_renderer.clone())
    }
}
