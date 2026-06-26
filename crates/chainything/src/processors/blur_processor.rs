use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::greyscale_processor::RawImage;

/// Applies a Gaussian blur to a [`RawImage`].
///
/// Takes an RGB image and a radius parameter (typically 1-5) and produces a blurred version.
/// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u32>` (radius)
/// - **Output:** one `Arc<RawImage>` with the same dimensions, blurred.
pub struct BlurProcessor {
    id: String,
    input_image: Option<Arc<RawImage>>,
    radius: Option<Arc<u32>>,
    output: Option<Arc<RawImage>>,
}

impl BlurProcessor {
    pub fn new(id: String) -> Self {
        BlurProcessor {
            id,
            input_image: None,
            radius: None,
            output: None,
        }
    }

    fn gaussian_blur(&self, image: &RawImage, radius_val: u32) -> RawImage {
        let radius = (radius_val as i32).max(1);
        let size = (2 * radius + 1) as usize;
        let sigma = radius as f32 / 3.0;

        let mut kernel = vec![0.0; size * size];
        let mut sum = 0.0;
        let pi2_sigma2 = 2.0 * std::f32::consts::PI * sigma * sigma;

        for y in 0..size {
            for x in 0..size {
                let dx = x as i32 - radius;
                let dy = y as i32 - radius;
                let val = (-(dx * dx + dy * dy) as f32 / (2.0 * sigma * sigma)).exp() / pi2_sigma2;
                kernel[y * size + x] = val;
                sum += val;
            }
        }

        for v in &mut kernel {
            *v /= sum;
        }

        let mut output_pixels = vec![0u8; image.pixels.len()];
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;
        let bytes_per_pixel = if is_rgb { 3 } else { 1 };

        for py in 0..image.height {
            for px in 0..image.width {
                for c in 0..bytes_per_pixel {
                    let mut sum = 0.0;
                    for ky in 0..size {
                        for kx in 0..size {
                            let iy = ((py as i32 + (ky as i32 - radius)).max(0) as u32)
                                .min(image.height - 1);
                            let ix = ((px as i32 + (kx as i32 - radius)).max(0) as u32)
                                .min(image.width - 1);
                            let idx = ((iy * image.width + ix) as usize * bytes_per_pixel) + c;
                            sum += image.pixels[idx] as f32 * kernel[ky * size + kx];
                        }
                    }
                    let out_idx = ((py * image.width + px) as usize * bytes_per_pixel) + c;
                    output_pixels[out_idx] = (sum.round() as u8).min(255);
                }
            }
        }

        RawImage {
            width: image.width,
            height: image.height,
            pixels: output_pixels,
        }
    }
}

impl Processor for BlurProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    /// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u32>` (radius)
    /// - **Errors:** [`ProcessorError::MissingInput`] if less than 2 inputs,
    ///   [`ProcessorError::InvalidInput`] if types don't match.
    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (image, radius), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let first_input = inputs.remove(0);
        let radius_input = inputs.remove(0);

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input_image = Some(typed_image);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for image (expected RawImage) for processor {}",
                self.id()
            )));
        }

        if let Ok(typed_radius) = radius_input.clone().downcast::<u32>() {
            self.radius = Some(typed_radius);
        } else if let Ok(typed_string) = radius_input.downcast::<String>() {
            let radius_val: u32 = typed_string
                .parse()
                .map_err(|_| {
                    ProcessorError::InvalidInput(format!(
                        "Cannot parse radius as u32 for processor {}",
                        self.id()
                    ))
                })?;
            self.radius = Some(Arc::new(radius_val));
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for radius (expected u32 or String) for processor {}",
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

        let radius = self
            .radius
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput(format!(
                "Missing radius for processor {}",
                self.id()
            )))?;

        let blurred = self.gaussian_blur(image, **radius);
        self.output = Some(Arc::new(blurred));
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
    fn test_blur_happy_path() {
        let image = create_test_image(3, 3, 100);
        let mut proc = BlurProcessor::new("blur".into());
        proc.set_input(vec![image, Arc::new(1u32)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        assert!(!output.is_empty());
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 3);
        assert_eq!(result.height, 3);
    }

    #[test]
    fn test_blur_without_image_returns_error() {
        let mut proc = BlurProcessor::new("blur".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_blur_missing_radius_returns_error() {
        let image = create_test_image(3, 3, 100);
        let mut proc = BlurProcessor::new("blur".into());
        let result = proc.set_input(vec![image]);
        assert!(matches!(result.unwrap_err(), ProcessorError::MissingInput(_)));
    }

    #[test]
    fn test_blur_wrong_image_type_returns_error() {
        let mut proc = BlurProcessor::new("blur".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new("not an image");
        let result = proc.set_input(vec![bad, Arc::new(1u32)]);
        assert!(matches!(result.unwrap_err(), ProcessorError::InvalidInput(_)));
    }

    #[test]
    fn test_blur_wrong_radius_type_returns_error() {
        let image = create_test_image(3, 3, 100);
        let mut proc = BlurProcessor::new("blur".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new("not a radius");
        let result = proc.set_input(vec![image, bad]);
        assert!(matches!(result.unwrap_err(), ProcessorError::InvalidInput(_)));
    }
}
