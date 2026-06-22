use std::collections::HashMap;
use serde_json::Value;

use crate::processors::{
    base_processor::ProcessorBase, 
    greyscale_processor::GreyScaleProcessor, 
    image_reader_processor::ImageReaderProcessor, 
    image_saver_processor::ImageSaveProcessor
};

type ProcessorConstructor = Box<dyn Fn(String, Value) -> Result<Box<dyn ProcessorBase>, String>>;

pub struct ProcessorRegistry {
    constructors: HashMap<String, ProcessorConstructor>,
}

impl ProcessorRegistry {
    pub fn new() -> Self {
        Self {
            constructors: HashMap::new(),
        }
    }

    /// Registers a new processor constructor under a specific node type name.
    ///
    /// This method takes a closure that defines how to construct the processor. The closure
    /// receives a unique `id` and any JSON `params` (from `serde_json::Value`) required for 
    /// initialization, allowing each processor to parse its own specific configuration.
    ///
    /// # Arguments
    ///
    /// * `node_type` - A string slice representing the name/type of the node (e.g., "InvertColor").
    /// * `constructor` - A closure that takes an `id` (`String`) and `params` (`Value`) and returns 
    ///   a `Result` containing the boxed `ProcessorBase`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut registry = ProcessorRegistry::new();
    /// 
    /// registry.register("InvertColor", |id| {
    ///     Ok(Box::new(InvertColorProcessor::new(id)))
    /// });
    /// ```
    pub fn register<F>(&mut self, node_type: &str, constructor: F)
    where
        F: Fn(String, Value) -> Result<Box<dyn ProcessorBase>, String> + 'static,
    {
        self.constructors.insert(
            node_type.to_string(),
            Box::new(constructor),
        );
    }

    pub fn build_processor(&self, node_type: &str, id: String, params: Value) -> Result<Box<dyn ProcessorBase>, String> {
        let constructor = self.constructors.get(node_type)
            .ok_or_else(|| format!("Unknown processor type: '{}'", node_type))?;
        
        constructor(id, params)
    }

    pub fn with_standard_processors() -> Self {
        let mut registry = Self::new();

        registry.register("ImageReader", |id, _params| {
            Ok(Box::new(ImageReaderProcessor::new(id)) as Box<dyn ProcessorBase>)
        });
        
        registry.register("Greyscale", |id, _params| {
            Ok(Box::new(GreyScaleProcessor::new(id)) as Box<dyn ProcessorBase>)
        });
        
        registry.register("ImageSave", |id, _params| {
            Ok(Box::new(ImageSaveProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self {
        Self::with_standard_processors()
    }
}