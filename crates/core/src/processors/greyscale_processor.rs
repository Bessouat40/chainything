use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};

#[derive(Clone)]
pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub struct GreyScaleProcessor {
    id: String,
    input: Option<Arc<RawImage>>,
    output: Option<Arc<RawImage>>,
}

impl GreyScaleProcessor {
    pub fn new(id: String) -> GreyScaleProcessor {
        GreyScaleProcessor {
            id,
            input: None,
            output: None,
        }
    }
}

impl Processor for GreyScaleProcessor {
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

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input = Some(typed_image);
            Ok(())
        } else {
            Err(ProcessorError::InvalidInput(format!(
                "Invalid input type (expected RawImage) for processor {}",
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
            let mut greyscale_image: Vec<u8> = Vec::with_capacity(input.pixels.len() / 3);

            for chunk in input.pixels.chunks(3) {
                if chunk.len() == 3 {
                    let value = 0.299 * (chunk[0] as f32)
                        + 0.587 * (chunk[1] as f32)
                        + 0.114 * (chunk[2] as f32);
                    greyscale_image.push(value as u8);
                }
            }

            self.output = Some(Arc::new(RawImage {
                width: input.width,
                height: input.height,
                pixels: greyscale_image,
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
