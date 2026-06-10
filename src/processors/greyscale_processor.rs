use crate::processors::base_processor::{Processor, RawImage};

pub struct GreyScaleProcessor;

impl Processor for GreyScaleProcessor {
    type Input = RawImage;
    type Output = RawImage;

    fn process(&self, input: &Self::Input, _parameters: Option<std::collections::HashMap<String, Box<dyn std::any::Any>>>) -> Self::Output {
        let mut greyscale_image: Vec<u8> = Vec::new();
        for chunk in input.pixels.chunks(3) {
            let value = 0.299 * (chunk[0] as f32)
                + 0.587 * (chunk[1] as f32)
                + 0.114 * (chunk[2] as f32);
            greyscale_image.push(value as u8);
        }
        return RawImage{
            width: input.width,
            height: input.height,
            pixels: greyscale_image
        }
    }
}