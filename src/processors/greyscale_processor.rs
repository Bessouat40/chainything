use std::{any::Any, sync::Arc};

use crate::processors::base_processor::{Processor, ProcessorError};
use std::any::{TypeId};

pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub struct GreyScaleProcessor {
    input: Option<Arc<RawImage>>,
    output: Option<Arc<RawImage>>
}

impl GreyScaleProcessor {
    pub fn new() -> GreyScaleProcessor {
            GreyScaleProcessor {
                input: None,
                output: None
            }
        }
}

impl Processor for GreyScaleProcessor {

    fn input_type(&self) -> TypeId {
        TypeId::of::<RawImage>()
    }

    fn output_type(&self) -> TypeId {
        TypeId::of::<RawImage>()
    }

    fn set_input(&mut self, input: Arc<dyn Any + Send + Sync>) {
        self.input = input.downcast::<RawImage>().ok();
    }

    fn get_output(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.output.clone().map(|o| o as Arc<dyn Any + Send + Sync>)
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
    
            self.output = Some(
                Arc::new(RawImage {
                width: input.width,
                height: input.height,
                pixels: greyscale_image,
            }));
    
            Ok(())
        } else {
            Err(ProcessorError::MissingInput)
        }
    }
}