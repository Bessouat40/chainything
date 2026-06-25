use crate::processors::{
    base_processor::{Processor, ProcessorError},
    greyscale_processor::RawImage,
};
use std::{any::Any, sync::Arc};

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
        if inputs.is_empty() {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (RawImage, path), got 0",
                self.id()
            )));
        }

        let first_input = inputs.remove(0);

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input = Some(typed_image);

            let second_input = inputs.remove(0);

            if let Ok(typed_path) = second_input.downcast::<String>() {
                self.output_path = Some(typed_path);
                Ok(())
            } else {
                Err(ProcessorError::InvalidInput(format!(
                    "Invalid second input type (expected String) for processor {}",
                    self.id()
                )))
            }
        } else {
            Err(ProcessorError::InvalidInput(format!(
                "Invalid input type (expected RawImage) for processor {}",
                self.id()
            )))
        }
    }

    fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        Vec::new()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let input = self.input.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!(
                "No input provided before running process on {}",
                self.id()
            ))
        })?;

        let output_path = self.output_path.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!(
                "No output path provided before running process on {}",
                self.id()
            ))
        })?;

        image::save_buffer(
            &output_path.as_ref(),
            &input.pixels,
            input.width,
            input.height,
            image::ColorType::L8,
        )
        .expect("An error occured trying to save your image...");

        Ok(())
    }
}
