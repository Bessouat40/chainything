use crate::processors::base_processor::{Processor, RawImage};

use image::GenericImageView;

pub struct ImageReaderProcessor;

impl Processor for ImageReaderProcessor {
    type Input = String;
    type Output = RawImage;

    fn process(&self, input: &Self::Input) -> Self::Output {
        let img = image::open(input).expect("An error occured trying to read your image...");
        let (width, height) = img.dimensions();
        
        let pixels = img.into_rgb8().into_raw(); 

        RawImage { width, height, pixels }
    }
}