use std::{any::{Any, TypeId}, sync::Arc};

use crate::processors::{base_processor::{Processor, ProcessorError}, greyscale_processor::RawImage};

use image::GenericImageView;

pub struct ImageReaderProcessor {
    input: Option<Arc<String>>,
    output: Option<Arc<RawImage>>
}

impl ImageReaderProcessor {
    pub fn new() -> ImageReaderProcessor {
        ImageReaderProcessor { input: None, output: None }
    }
}

impl Processor for ImageReaderProcessor {

    fn set_input(&mut self, input: Arc<dyn Any + Send + Sync>) {
        self.input = input.downcast::<String>().ok();
    }

    fn get_output(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.output.clone().map(|o| o as Arc<dyn Any + Send + Sync>)
    }

    fn input_type(&self) -> TypeId {
        TypeId::of::<String>()
    }

    fn output_type(&self) -> TypeId {
        TypeId::of::<RawImage>()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        if let Some(input) = &self.input {
            let img = image::open(input.as_ref())
                .map_err(|_| ProcessorError::ComputingError("An error occured reading your image...".to_string()))?;
            let (width, height) = img.dimensions();
    
            let pixels = img.into_rgb8().into_raw();
    
            self.output = Some(Arc::new(RawImage {
                width,
                height,
                pixels
            }));
            
            Ok(())
        } else {
            Err(ProcessorError::MissingInput)
        }
    }
}