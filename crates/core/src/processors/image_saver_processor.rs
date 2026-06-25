use crate::processors::{
    base_processor::{Processor, ProcessorError},
    greyscale_processor::RawImage,
};
use std::{any::Any, sync::Arc};

/// The `ImageSaveProcessor` saves a `RawImage` to the filesystem.
///
/// ### Input
/// * Expects two inputs: 
///   1. `Arc<RawImage>` (the image data).
///   2. `Arc<String>` (the file path to save to).
///
/// ### Output
/// * Returns an empty vector (no data output).
///
/// ### Errors
/// * Returns `ProcessorError::MissingInput` if inputs are not provided.
/// * Returns `ProcessorError::InvalidInput` if types are mismatched.
pub struct ImageSaveProcessor {
    id: String,
    input: Option<Arc<RawImage>>,
    output_path: Option<Arc<String>>,
}

impl ImageSaveProcessor {
    pub fn new(id: String) -> Self {
        ImageSaveProcessor {
            id,
            input: None,
            output_path: None,
        }
    }
}

impl Processor for ImageSaveProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (RawImage, path), got {}",
                self.id(), inputs.len()
            )));
        }

        let first = inputs.remove(0);
        let second = inputs.remove(0);

        self.input = Some(first.downcast::<RawImage>().map_err(|_| {
            ProcessorError::InvalidInput("First input must be RawImage".to_string())
        })?);

        self.output_path = Some(second.downcast::<String>().map_err(|_| {
            ProcessorError::InvalidInput("Second input must be String".to_string())
        })?);

        Ok(())
    }

    fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        Vec::new()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let input = self.input.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput("Missing image input".to_string())
        })?;

        let output_path = self.output_path.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput("Missing output path".to_string())
        })?;

        image::save_buffer(
            output_path.as_str(),
            &input.pixels,
            input.width,
            input.height,
            image::ColorType::L8,
        )
        .map_err(|e| ProcessorError::ComputingError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_inputs_fails() {
        let mut processor = ImageSaveProcessor::new("save_test".to_string());
        assert!(processor.set_input(vec![]).is_err());
    }

    #[test]
    fn test_invalid_input_types_fails() {
        let mut processor = ImageSaveProcessor::new("save_test".to_string());
        let inputs: Vec<Arc<dyn Any + Send + Sync>> = vec![Arc::new("path".to_string()), Arc::new(10)];
        assert!(processor.set_input(inputs).is_err());
    }

    #[test]
    fn test_process_without_input_fails() {
        let mut processor = ImageSaveProcessor::new("save_test".to_string());
        assert!(processor.process().is_err());
    }
}