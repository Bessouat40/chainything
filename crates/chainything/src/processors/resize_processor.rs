use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::greyscale_processor::RawImage;

/// Resizes a [`RawImage`] to new dimensions using nearest-neighbor interpolation.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u32>` (new width), `inputs[2]` = `Arc<u32>` (new height)
/// - **Output:** one `Arc<RawImage>` with the new dimensions.
pub struct ResizeProcessor {
    id: String,
    input_image: Option<Arc<RawImage>>,
    new_width: Option<Arc<u32>>,
    new_height: Option<Arc<u32>>,
    output: Option<Arc<RawImage>>,
}

impl ResizeProcessor {
    pub fn new(id: String) -> Self {
        ResizeProcessor {
            id,
            input_image: None,
            new_width: None,
            new_height: None,
            output: None,
        }
    }

    fn resize_nearest_neighbor(&self, image: &RawImage, new_width: u32, new_height: u32) -> RawImage {
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;
        let bytes_per_pixel = if is_rgb { 3 } else { 1 };

        let mut output_pixels = vec![0u8; (new_width * new_height * bytes_per_pixel as u32) as usize];

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (((x as f32 / new_width as f32) * image.width as f32) as u32)
                    .min(image.width - 1);
                let src_y = (((y as f32 / new_height as f32) * image.height as f32) as u32)
                    .min(image.height - 1);

                for c in 0..bytes_per_pixel {
                    let src_idx = ((src_y * image.width + src_x) as usize * bytes_per_pixel) + c;
                    let dst_idx = ((y * new_width + x) as usize * bytes_per_pixel) + c;
                    output_pixels[dst_idx] = image.pixels[src_idx];
                }
            }
        }

        RawImage {
            width: new_width,
            height: new_height,
            pixels: output_pixels,
        }
    }
}

impl Processor for ResizeProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    /// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u32>` (new width), `inputs[2]` = `Arc<u32>` (new height)
    /// - **Errors:** [`ProcessorError::MissingInput`] if less than 3 inputs,
    ///   [`ProcessorError::InvalidInput`] if types don't match.
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

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input_image = Some(typed_image);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for image (expected RawImage) for processor {}",
                self.id()
            )));
        }

        if let Ok(typed_width) = width_input.clone().downcast::<u32>() {
            self.new_width = Some(typed_width);
        } else if let Ok(typed_string) = width_input.downcast::<String>() {
            let width_val: u32 = typed_string
                .parse()
                .map_err(|_| {
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
            let height_val: u32 = typed_string
                .parse()
                .map_err(|_| {
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
        let image = self
            .input_image
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput(format!(
                "Missing input image for processor {}",
                self.id()
            )))?;

        let width = self
            .new_width
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput(format!(
                "Missing width for processor {}",
                self.id()
            )))?;

        let height = self
            .new_height
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput(format!(
                "Missing height for processor {}",
                self.id()
            )))?;

        if **width == 0 || **height == 0 {
            return Err(ProcessorError::ComputingError(
                "Width and height must be greater than 0".into(),
            ));
        }

        let resized = self.resize_nearest_neighbor(image, **width, **height);
        self.output = Some(Arc::new(resized));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, color: u8) -> Arc<RawImage> {
        let pixels = vec![color; (width * height * 3) as usize];
        Arc::new(RawImage {
            width,
            height,
            pixels,
        })
    }

    #[test]
    fn test_resize_happy_path() {
        let image = create_test_image(4, 4, 100);
        let mut proc = ResizeProcessor::new("resize".into());
        proc.set_input(vec![image, Arc::new(2u32), Arc::new(2u32)])
            .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        assert!(!output.is_empty());
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 2);
        assert_eq!(result.pixels.len(), 2 * 2 * 3);
    }

    #[test]
    fn test_resize_upscale() {
        let image = create_test_image(2, 2, 100);
        let mut proc = ResizeProcessor::new("resize".into());
        proc.set_input(vec![image, Arc::new(4u32), Arc::new(4u32)])
            .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);
    }

    #[test]
    fn test_resize_without_image_returns_error() {
        let mut proc = ResizeProcessor::new("resize".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_resize_missing_dimensions_returns_error() {
        let image = create_test_image(4, 4, 100);
        let mut proc = ResizeProcessor::new("resize".into());
        let result = proc.set_input(vec![image]);
        assert!(matches!(result.unwrap_err(), ProcessorError::MissingInput(_)));
    }

    #[test]
    fn test_resize_zero_dimension_returns_error() {
        let image = create_test_image(4, 4, 100);
        let mut proc = ResizeProcessor::new("resize".into());
        proc.set_input(vec![image, Arc::new(0u32), Arc::new(4u32)])
            .unwrap();
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::ComputingError(_)
        ));
    }

    #[test]
    fn test_resize_wrong_image_type_returns_error() {
        let mut proc = ResizeProcessor::new("resize".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new("not an image");
        let result = proc.set_input(vec![bad, Arc::new(2u32), Arc::new(2u32)]);
        assert!(matches!(result.unwrap_err(), ProcessorError::InvalidInput(_)));
    }
}
