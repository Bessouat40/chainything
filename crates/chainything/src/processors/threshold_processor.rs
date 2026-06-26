use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::greyscale_processor::RawImage;

/// Binarizes a [`RawImage`] using a threshold value.
///
/// Converts each pixel to black (0) or white (255) based on a threshold.
/// For RGB images, uses the luminosity formula: Y = 0.299R + 0.587G + 0.114B
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u8>` (threshold, 0-255)
/// - **Output:** one `Arc<RawImage>` with binary pixels (one byte per pixel).
pub struct ThresholdProcessor {
    id: String,
    input_image: Option<Arc<RawImage>>,
    threshold: Option<Arc<u8>>,
    output: Option<Arc<RawImage>>,
}

impl ThresholdProcessor {
    pub fn new(id: String) -> Self {
        ThresholdProcessor {
            id,
            input_image: None,
            threshold: None,
            output: None,
        }
    }

    fn apply_threshold(&self, image: &RawImage, threshold: u8) -> RawImage {
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;
        let mut output_pixels = Vec::new();

        if is_rgb {
            for chunk in image.pixels.chunks(3) {
                if chunk.len() == 3 {
                    let luminosity = (0.299 * chunk[0] as f32
                        + 0.587 * chunk[1] as f32
                        + 0.114 * chunk[2] as f32) as u8;
                    let binary = if luminosity >= threshold { 255 } else { 0 };
                    output_pixels.push(binary);
                }
            }
        } else {
            for &pixel in &image.pixels {
                let binary = if pixel >= threshold { 255 } else { 0 };
                output_pixels.push(binary);
            }
        }

        RawImage {
            width: image.width,
            height: image.height,
            pixels: output_pixels,
        }
    }
}

impl Processor for ThresholdProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    /// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u8>` (threshold)
    /// - **Errors:** [`ProcessorError::MissingInput`] if less than 2 inputs,
    ///   [`ProcessorError::InvalidInput`] if types don't match.
    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (image, threshold), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let first_input = inputs.remove(0);
        let threshold_input = inputs.remove(0);

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input_image = Some(typed_image);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for image (expected RawImage) for processor {}",
                self.id()
            )));
        }

        if let Ok(typed_threshold) = threshold_input.clone().downcast::<u8>() {
            self.threshold = Some(typed_threshold);
        } else if let Ok(typed_string) = threshold_input.downcast::<String>() {
            let threshold_val: u8 = typed_string
                .parse()
                .map_err(|_| {
                    ProcessorError::InvalidInput(format!(
                        "Cannot parse threshold as u8 for processor {}",
                        self.id()
                    ))
                })?;
            self.threshold = Some(Arc::new(threshold_val));
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for threshold (expected u8 or String) for processor {}",
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

        let threshold = self
            .threshold
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput(format!(
                "Missing threshold for processor {}",
                self.id()
            )))?;

        let thresholded = self.apply_threshold(image, **threshold);
        self.output = Some(Arc::new(thresholded));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, pixels: Vec<u8>) -> Arc<RawImage> {
        Arc::new(RawImage {
            width,
            height,
            pixels,
        })
    }

    #[test]
    fn test_threshold_happy_path_rgb() {
        let image = create_test_image(1, 2, vec![255, 0, 0, 0, 0, 0]);
        let mut proc = ThresholdProcessor::new("threshold".into());
        proc.set_input(vec![image, Arc::new(128u8)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        assert!(!output.is_empty());
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 2);
        assert_eq!(result.pixels.len(), 2);
    }

    #[test]
    fn test_threshold_binarization_rgb() {
        let image = create_test_image(2, 1, vec![255, 0, 0, 0, 0, 0]);
        let mut proc = ThresholdProcessor::new("threshold".into());
        proc.set_input(vec![image, Arc::new(100u8)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        let expected_first = if (0.299 * 255.0) as u8 >= 100 { 255 } else { 0 };
        let expected_second = if (0.299 * 0.0 + 0.587 * 0.0 + 0.114 * 0.0) as u8 >= 100 {
            255
        } else {
            0
        };
        assert_eq!(result.pixels[0], expected_first);
        assert_eq!(result.pixels[1], expected_second);
    }

    #[test]
    fn test_threshold_greyscale() {
        let image = create_test_image(2, 1, vec![200, 100]);
        let mut proc = ThresholdProcessor::new("threshold".into());
        proc.set_input(vec![image, Arc::new(150u8)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.pixels[0], 255);
        assert_eq!(result.pixels[1], 0);
    }

    #[test]
    fn test_threshold_without_image_returns_error() {
        let mut proc = ThresholdProcessor::new("threshold".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_threshold_missing_threshold_returns_error() {
        let image = create_test_image(2, 1, vec![200, 100]);
        let mut proc = ThresholdProcessor::new("threshold".into());
        let result = proc.set_input(vec![image]);
        assert!(matches!(result.unwrap_err(), ProcessorError::MissingInput(_)));
    }

    #[test]
    fn test_threshold_wrong_image_type_returns_error() {
        let mut proc = ThresholdProcessor::new("threshold".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new("not an image");
        let result = proc.set_input(vec![bad, Arc::new(128u8)]);
        assert!(matches!(result.unwrap_err(), ProcessorError::InvalidInput(_)));
    }

    #[test]
    fn test_threshold_wrong_threshold_type_returns_error() {
        let image = create_test_image(2, 1, vec![200, 100]);
        let mut proc = ThresholdProcessor::new("threshold".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new("not a threshold");
        let result = proc.set_input(vec![image, bad]);
        assert!(matches!(result.unwrap_err(), ProcessorError::InvalidInput(_)));
    }
}
