use std::sync::Arc;

use crate::processors::base_processor::Processor;

mod processors;
use image::ColorType;
use processors::image_reader_processor::ImageReaderProcessor;
use processors::greyscale_processor::{RawImage, GreyScaleProcessor};

mod dag;


fn main() {
    let image_path = String::from("./chat.jpg");
    let mut reader = ImageReaderProcessor::new();
    reader.set_input(Arc::new(image_path));
    let _ = reader.process().expect("Failed to read image");
    let mut greyscale_processor = GreyScaleProcessor::new();
    greyscale_processor.set_input(reader.get_output().unwrap());
    let _ = greyscale_processor.process();
    let greyscale_image = greyscale_processor
        .get_output()
        .unwrap()
        .downcast::<RawImage>()
        .unwrap();
    image::save_buffer(
        "output.jpg", 
        &greyscale_image.pixels, 
        greyscale_image.width, 
        greyscale_image.height, 
        ColorType::L8
    ).expect("An error occured trying to save your image...");
}
