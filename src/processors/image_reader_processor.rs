use std::{sync::Arc};

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

    type Input = String;
    type Output = RawImage;

    fn set_input(&mut self, input: Arc<String>) {
        self.input = Some(input);
    }

    fn get_output(&self) -> Option<Arc<RawImage>> {
        self.output.clone()

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