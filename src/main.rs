use image::{ColorType};

use crate::processors::base_processor::Processor;

mod processors;
use processors::image_reader_processor::ImageReaderProcessor;
use processors::greyscale_processor::GreyScaleProcessor;


fn main() {
    let image_path = String::from("./chat.jpg");
    let reader = ImageReaderProcessor;
    let image = reader.process(&image_path);
    let greyscale_processor = GreyScaleProcessor;
    let greyscale_image = greyscale_processor.process(&image);
    image::save_buffer(
        "output.jpg", 
        &greyscale_image.pixels, 
        greyscale_image.width, 
        greyscale_image.height, 
        ColorType::L8
    ).expect("An error occured trying to save your image...");
}
