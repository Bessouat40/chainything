use image::{GrayImage, RgbImage};
use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;

/// Adjusts the brightness of a [`RawImage`] by a signed delta.
///
/// Every channel byte is shifted by `delta` and clamped to the `0..=255` range.
/// A positive delta brightens the image, a negative delta darkens it. Works on
/// both RGB and greyscale images.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<i32>` /
///   `Arc<u32>` / `Arc<String>` (delta, may be negative).
/// - **Output:** one `Arc<RawImage>` with the same dimensions and layout.
/// - **Errors:** [`ProcessorError::MissingInput`] if fewer than 2 inputs,
///   [`ProcessorError::InvalidInput`] if a type is wrong or the delta cannot be
///   parsed.
pub struct BrightnessProcessor {
    id: String,
    input_image: Option<Arc<RawImage>>,
    delta: Option<i32>,
    output: Option<Arc<RawImage>>,
}

impl BrightnessProcessor {
    pub fn new(id: String) -> Self {
        BrightnessProcessor {
            id,
            input_image: None,
            delta: None,
            output: None,
        }
    }

    fn brighten(image: &RawImage, delta: i32) -> RawImage {
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;

        let pixels = if is_rgb {
            match RgbImage::from_raw(image.width, image.height, image.pixels.clone()) {
                Some(buf) => image::imageops::colorops::brighten(&buf, delta).into_raw(),
                None => image.pixels.clone(),
            }
        } else {
            match GrayImage::from_raw(image.width, image.height, image.pixels.clone()) {
                Some(buf) => image::imageops::colorops::brighten(&buf, delta).into_raw(),
                None => image.pixels.clone(),
            }
        };

        RawImage {
            width: image.width,
            height: image.height,
            pixels,
        }
    }
}

impl Processor for BrightnessProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (image, delta), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let first_input = inputs.remove(0);
        let delta_input = inputs.remove(0);

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input_image = Some(typed_image);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for image (expected RawImage) for processor {}",
                self.id()
            )));
        }

        let delta_val = if let Ok(typed_delta) = delta_input.clone().downcast::<i32>() {
            *typed_delta
        } else if let Ok(typed_u32) = delta_input.clone().downcast::<u32>() {
            *typed_u32 as i32
        } else if let Ok(typed_string) = delta_input.downcast::<String>() {
            typed_string.parse().map_err(|_| {
                ProcessorError::InvalidInput(format!(
                    "Cannot parse delta as i32 for processor {}",
                    self.id()
                ))
            })?
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for delta (expected i32, u32 or String) for processor {}",
                self.id()
            )));
        };

        self.delta = Some(delta_val);
        Ok(())
    }

    fn get_output(&self) -> Vec<Arc<dyn std::any::Any + Send + Sync>> {
        self.output
            .clone()
            .into_iter()
            .map(|out| out as Arc<dyn std::any::Any + Send + Sync>)
            .collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let image = self.input_image.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing input image for processor {}", self.id()))
        })?;

        let delta = self.delta.ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing delta for processor {}", self.id()))
        })?;

        let brightened = Self::brighten(image, delta);
        self.output = Some(Arc::new(brightened));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_gray_image(pixels: Vec<u8>) -> Arc<RawImage> {
        Arc::new(RawImage {
            width: pixels.len() as u32,
            height: 1,
            pixels,
        })
    }

    fn run(image: Arc<RawImage>, delta: Arc<dyn std::any::Any + Send + Sync>) -> Vec<u8> {
        let mut proc = BrightnessProcessor::new("brightness".into());
        proc.set_input(vec![image, delta]).unwrap();
        proc.process().unwrap();
        let output = proc.get_output();
        output[0].downcast_ref::<RawImage>().unwrap().pixels.clone()
    }

    #[test]
    fn test_brighten_positive_clamps_at_255() {
        let image = create_gray_image(vec![100, 250]);
        assert_eq!(run(image, Arc::new(50u32)), vec![150, 255]);
    }

    #[test]
    fn test_darken_negative_clamps_at_zero() {
        let image = create_gray_image(vec![100, 20]);
        assert_eq!(run(image, Arc::new(-40i32)), vec![60, 0]);
    }

    #[test]
    fn test_brighten_from_string_delta() {
        let image = create_gray_image(vec![100]);
        assert_eq!(run(image, Arc::new("-10".to_string())), vec![90]);
    }

    #[test]
    fn test_brightness_without_image_returns_error() {
        let mut proc = BrightnessProcessor::new("brightness".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_brightness_missing_delta_returns_error() {
        let image = create_gray_image(vec![100]);
        let mut proc = BrightnessProcessor::new("brightness".into());
        assert!(matches!(
            proc.set_input(vec![image]).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_brightness_wrong_image_type_returns_error() {
        let mut proc = BrightnessProcessor::new("brightness".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new(1u32);
        assert!(matches!(
            proc.set_input(vec![bad, Arc::new(10i32)]).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }
}
