use std::sync::Arc;

use crate::processors::{
    base_processor::{Processor, ProcessorError},
    greyscale_processor::RawImage,
};

use image::GenericImageView;

pub struct ImageReaderProcessor {
    input: Option<Arc<String>>,
    output: Option<Arc<RawImage>>,
    id: String,
}

impl ImageReaderProcessor {
    pub fn new(id: String) -> ImageReaderProcessor {
        ImageReaderProcessor {
            input: None,
            output: None,
            id,
        }
    }
}

impl Processor for ImageReaderProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.is_empty() {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 1 input, got 0",
                self.id()
            )));
        }

        let first_input = inputs.remove(0);

        if let Ok(typed_input) = first_input.downcast::<String>() {
            self.input = Some(typed_input);
            Ok(())
        } else {
            Err(ProcessorError::InvalidInput(format!(
                "Invalid input type (expected String) for processor {}",
                self.id()
            )))
        }
    }

    fn get_output(&self) -> Vec<Arc<dyn std::any::Any + Send + Sync>> {
        self.output
            .clone()
            .into_iter()
            .map(|out| out as Arc<dyn std::any::Any + Send + Sync>)
            .collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        if let Some(input) = &self.input {
            let img = image::open(input.as_ref()).map_err(|_| {
                ProcessorError::ComputingError("An error occured reading your image...".to_string())
            })?;
            let (width, height) = img.dimensions();

            let pixels = img.into_rgb8().into_raw();

            self.output = Some(Arc::new(RawImage {
                width,
                height,
                pixels,
            }));

            Ok(())
        } else {
            Err(ProcessorError::MissingInput(format!(
                "Missing input for processor {}",
                self.id()
            )))
        }
    }
}
