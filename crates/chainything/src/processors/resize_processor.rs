use image::{DynamicImage, imageops::FilterType};
use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::greyscale_processor::RawImage;

/// Resizes an image using the `image` crate.
///
/// - **Input:** `inputs[0]` = `Arc<DynamicImage>` or `Arc<RawImage>`, `inputs[1]` = `Arc<u32>` (width), `inputs[2]` = `Arc<u32>` (height)
/// - **Output:** One `Arc<RawImage>` containing the resized image.
pub struct ImageResizeProcessor {
    id: String,
    input_image: Option<Arc<DynamicImage>>,
    new_width: Option<Arc<u32>>,
    new_height: Option<Arc<u32>>,
    output: Option<Arc<RawImage>>,
}

impl ImageResizeProcessor {
    pub fn new(id: String) -> Self {
        ImageResizeProcessor {
            id,
            input_image: None,
            new_width: None,
            new_height: None,
            output: None,
        }
    }
}

impl Processor for ImageResizeProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 3 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 3 inputs (image, width, height), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let first_input = inputs.remove(0);
        let width_input = inputs.remove(0);
        let height_input = inputs.remove(0);

        if let Ok(typed_image) = first_input.clone().downcast::<DynamicImage>() {
            self.input_image = Some(typed_image);
        } else if let Ok(raw_image) = first_input.downcast::<RawImage>() {
            let is_rgb =
                raw_image.pixels.len() == (raw_image.width * raw_image.height * 3) as usize;
            let dynamic_img = if is_rgb {
                let rgb_buf = image::RgbImage::from_raw(
                    raw_image.width,
                    raw_image.height,
                    raw_image.pixels.clone(),
                )
                .ok_or_else(|| ProcessorError::InvalidInput("Invalid RGB buffer".into()))?;
                DynamicImage::ImageRgb8(rgb_buf)
            } else {
                let gray_buf = image::GrayImage::from_raw(
                    raw_image.width,
                    raw_image.height,
                    raw_image.pixels.clone(),
                )
                .ok_or_else(|| ProcessorError::InvalidInput("Invalid Grayscale buffer".into()))?;
                DynamicImage::ImageLuma8(gray_buf)
            };
            self.input_image = Some(Arc::new(dynamic_img));
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for image (expected DynamicImage or RawImage) for processor {}",
                self.id()
            )));
        }

        if let Ok(typed_width) = width_input.clone().downcast::<u32>() {
            self.new_width = Some(typed_width);
        } else if let Ok(typed_string) = width_input.downcast::<String>() {
            let width_val: u32 = typed_string.parse().map_err(|_| {
                ProcessorError::InvalidInput(format!(
                    "Cannot parse width as u32 for processor {}",
                    self.id()
                ))
            })?;
            self.new_width = Some(Arc::new(width_val));
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for width (expected u32 or String) for processor {}",
                self.id()
            )));
        }

        if let Ok(typed_height) = height_input.clone().downcast::<u32>() {
            self.new_height = Some(typed_height);
        } else if let Ok(typed_string) = height_input.downcast::<String>() {
            let height_val: u32 = typed_string.parse().map_err(|_| {
                ProcessorError::InvalidInput(format!(
                    "Cannot parse height as u32 for processor {}",
                    self.id()
                ))
            })?;
            self.new_height = Some(Arc::new(height_val));
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for height (expected u32 or String) for processor {}",
                self.id()
            )));
        }

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

        let width = self.new_width.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing width for processor {}", self.id()))
        })?;

        let height = self.new_height.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing height for processor {}", self.id()))
        })?;

        if **width == 0 || **height == 0 {
            return Err(ProcessorError::ComputingError(
                "Width and height must be greater than 0".into(),
            ));
        }

        let resized = image.resize_exact(**width, **height, FilterType::Nearest);

        // Convert back to RawImage so downstream processors (which all share the
        // RawImage contract) can consume the output. Preserve the grayscale vs RGB
        // layout the saver's pixel-count heuristic relies on.
        let raw = match &resized {
            DynamicImage::ImageLuma8(_) => {
                let buf = resized.to_luma8();
                RawImage {
                    width: buf.width(),
                    height: buf.height(),
                    pixels: buf.into_raw(),
                }
            }
            _ => {
                let buf = resized.to_rgb8();
                RawImage {
                    width: buf.width(),
                    height: buf.height(),
                    pixels: buf.into_raw(),
                }
            }
        };

        self.output = Some(Arc::new(raw));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    fn create_test_image(width: u32, height: u32) -> Arc<DynamicImage> {
        let rgb = RgbImage::from_pixel(width, height, image::Rgb([255, 0, 0]));
        Arc::new(DynamicImage::ImageRgb8(rgb))
    }

    #[test]
    fn test_resize_happy_path() {
        let image = create_test_image(4, 4);
        let mut proc = ImageResizeProcessor::new("resize".into());
        proc.set_input(vec![image, Arc::new(2u32), Arc::new(2u32)])
            .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        assert!(!output.is_empty());

        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 2);
    }

    #[test]
    fn test_resize_zero_dimension_returns_error() {
        let image = create_test_image(4, 4);
        let mut proc = ImageResizeProcessor::new("resize".into());
        proc.set_input(vec![image, Arc::new(0u32), Arc::new(4u32)])
            .unwrap();
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::ComputingError(_)
        ));
    }

    #[test]
    fn test_resize_wrong_image_type_returns_error() {
        let mut proc = ImageResizeProcessor::new("resize".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new("not an image");
        let result = proc.set_input(vec![bad, Arc::new(2u32), Arc::new(2u32)]);
        assert!(matches!(
            result.unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }
}
