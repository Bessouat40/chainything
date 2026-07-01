use std::collections::HashMap;

use crate::processors::{
    images::{
        blur_processor::BlurProcessor, brightness_processor::BrightnessProcessor,
        edge_detect_processor::EdgeDetectProcessor, greyscale_processor::GreyScaleProcessor,
        image_reader_processor::ImageReaderProcessor, image_saver_processor::ImageSaveProcessor,
        invert_processor::InvertProcessor, resize_processor::ImageResizeProcessor,
        rotate_processor::RotateProcessor, threshold_processor::ImageThresholdProcessor,
    },
    llm::{
        llm_generate_processor::LlmGenerateProcessor,
        ollama_loader_processor::OllamaLoaderProcessor,
    },
    model3d::{
        model_reader_processor::ModelReaderProcessor, model_render_processor::ModelRenderProcessor,
        model_saver_processor::ModelSaveProcessor, model_scale_processor::ModelScaleProcessor,
    },
    text::text_saver_processor::TextSaveProcessor,
};

use crate::processors::base_processor::ProcessorBase;

/// Type alias for a function/closure that creates a boxed processor instance.
type ProcessorConstructor = Box<dyn Fn(String) -> Result<Box<dyn ProcessorBase>, String>>;

/// A registry responsible for mapping node type strings to processor factory functions.
///
/// This acts as a factory pattern container, allowing for dynamic instantiation of
/// different processor types based on configuration.
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
    /// use chainything::prelude::ProcessorRegistry;
    /// use chainything::prelude::GreyScaleProcessor;
    /// let mut registry = ProcessorRegistry::new();
    ///
    /// registry.register("InvertColor", |id| {
    ///     Ok(Box::new(GreyScaleProcessor::new(id)))
    /// });
    /// ```
    pub fn register<F>(&mut self, node_type: &str, constructor: F)
    where
        F: Fn(String) -> Result<Box<dyn ProcessorBase>, String> + 'static,
    {
        self.constructors
            .insert(node_type.to_string(), Box::new(constructor));
    }

    /// Instantiates a processor of the specified `node_type`.
    ///
    /// # Arguments
    /// * `node_type` - The type identifier to look up in the registry.
    /// * `id` - The unique identifier to assign to the new processor instance.
    ///
    /// # Returns
    /// * `Ok(Box<dyn ProcessorBase>)` - The successfully created processor.
    /// * `Err(String)` - An error message if the `node_type` is unknown or construction fails.
    pub fn build_processor(
        &self,
        node_type: &str,
        id: String,
    ) -> Result<Box<dyn ProcessorBase>, String> {
        let constructor = self
            .constructors
            .get(node_type)
            .ok_or_else(|| format!("Unknown processor type: '{}'", node_type))?;

        constructor(id)
    }

    pub fn with_standard_processors() -> Self {
        let mut registry = Self::new();

        registry.register("ImageReader", |id| {
            Ok(Box::new(ImageReaderProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("Greyscale", |id| {
            Ok(Box::new(GreyScaleProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("ImageSave", |id| {
            Ok(Box::new(ImageSaveProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("Blur", |id| {
            Ok(Box::new(BlurProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("Resize", |id| {
            Ok(Box::new(ImageResizeProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("Threshold", |id| {
            Ok(Box::new(ImageThresholdProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("Invert", |id| {
            Ok(Box::new(InvertProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("Rotate", |id| {
            Ok(Box::new(RotateProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("Brightness", |id| {
            Ok(Box::new(BrightnessProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("EdgeDetect", |id| {
            Ok(Box::new(EdgeDetectProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("OllamaLoader", |id| {
            Ok(Box::new(OllamaLoaderProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("LLMGenerate", |id| {
            Ok(Box::new(LlmGenerateProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("TextSave", |id| {
            Ok(Box::new(TextSaveProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("ModelReader", |id| {
            Ok(Box::new(ModelReaderProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("ModelScale", |id| {
            Ok(Box::new(ModelScaleProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("ModelSave", |id| {
            Ok(Box::new(ModelSaveProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry.register("ModelRender", |id| {
            Ok(Box::new(ModelRenderProcessor::new(id)) as Box<dyn ProcessorBase>)
        });

        registry
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self {
        Self::with_standard_processors()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::base_processor::{Processor, ProcessorError};
    use std::{any::Any, sync::Arc};

    struct AddOneProcessor {
        id: String,
        input: Vec<i32>,
        output: Vec<i32>,
    }

    impl AddOneProcessor {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                input: vec![],
                output: vec![],
            }
        }
    }

    impl Processor for AddOneProcessor {
        fn id(&self) -> &str {
            &self.id
        }

        fn set_input(
            &mut self,
            input: Vec<Arc<dyn Any + Send + Sync>>,
        ) -> Result<(), ProcessorError> {
            self.input = input
                .iter()
                .map(|v| {
                    v.downcast_ref::<i32>()
                        .copied()
                        .ok_or_else(|| ProcessorError::InvalidInput("Expected i32".to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(())
        }

        fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
            self.output
                .iter()
                .map(|&v| Arc::new(v) as Arc<dyn Any + Send + Sync>)
                .collect()
        }

        fn process(&mut self) -> Result<(), ProcessorError> {
            if self.input.is_empty() {
                return Err(ProcessorError::MissingInput("No input".to_string()));
            }
            self.output = self.input.iter().map(|&x| x + 1).collect();
            Ok(())
        }
    }

    fn arc_i32(v: i32) -> Arc<dyn Any + Send + Sync> {
        Arc::new(v) as Arc<dyn Any + Send + Sync>
    }

    #[test]
    fn test_nominal_pipeline() {
        let mut p = AddOneProcessor::new("p1");
        p.set_input_erased(vec![arc_i32(1), arc_i32(9)]).unwrap();
        Processor::process(&mut p).unwrap();

        let out = p.get_output_erased();
        assert_eq!(*out[0].downcast_ref::<i32>().unwrap(), 2);
        assert_eq!(*out[1].downcast_ref::<i32>().unwrap(), 10);
    }

    #[test]
    fn test_set_input_wrong_type() {
        let mut p = AddOneProcessor::new("p1");
        let bad = Arc::new("nope".to_string()) as Arc<dyn Any + Send + Sync>;
        assert!(matches!(
            p.set_input_erased(vec![bad]),
            Err(ProcessorError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_process_missing_input() {
        let mut p = AddOneProcessor::new("p1");
        assert!(matches!(
            Processor::process(&mut p),
            Err(ProcessorError::MissingInput(_))
        ));
    }

    #[test]
    fn test_id_and_send_sync() {
        let p = AddOneProcessor::new("my-id");
        let base: &dyn ProcessorBase = &p;
        assert_eq!(base.id(), "my-id");
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AddOneProcessor>();
    }
}
