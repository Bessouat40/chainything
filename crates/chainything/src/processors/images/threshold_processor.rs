use image::DynamicImage;
use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;

/// Binarizes an image using a threshold value (0-255).
///
/// Converts each pixel to black (0) or white (255) based on a threshold.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u8>` (threshold)
/// - **Output:** One `Arc<RawImage>` containing the binary image (Grayscale format).
pub struct ImageThresholdProcessor {
    id: String,
    input_image: Option<Arc<RawImage>>,
    threshold: Option<Arc<u8>>,
    output: Option<Arc<RawImage>>,
}

impl ImageThresholdProcessor {
    pub fn new(id: String) -> Self {
        ImageThresholdProcessor {
            id,
            input_image: None,
            threshold: None,
            output: None,
        }
    }
}

impl Processor for ImageThresholdProcessor {
    fn id(&self) -> &str {
        &self.id
    }

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

        // Correction : On attend un RawImage, comme envoyé par ImageReader
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
        } else if let Ok(typed_u32) = threshold_input.clone().downcast::<u32>() {
            self.threshold = Some(Arc::new(*typed_u32 as u8));
        } else if let Ok(typed_string) = threshold_input.downcast::<String>() {
            let threshold_val: u8 = typed_string.parse().map_err(|_| {
                ProcessorError::InvalidInput(format!(
                    "Cannot parse threshold as u8 for processor {}",
                    self.id()
                ))
            })?;
            self.threshold = Some(Arc::new(threshold_val));
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for threshold (expected u8, u32 or String) for processor {}",
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

        let threshold = self.threshold.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing threshold for processor {}", self.id()))
        })?;

        // 1. Reconstruire le DynamicImage depuis les pixels bruts du RawImage
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;

        let dynamic_img = if is_rgb {
            let rgb_buf =
                image::RgbImage::from_raw(image.width, image.height, image.pixels.clone())
                    .ok_or_else(|| ProcessorError::ComputingError("Invalid RGB buffer".into()))?;
            DynamicImage::ImageRgb8(rgb_buf)
        } else {
            let gray_buf =
                image::GrayImage::from_raw(image.width, image.height, image.pixels.clone())
                    .ok_or_else(|| {
                        ProcessorError::ComputingError("Invalid Grayscale buffer".into())
                    })?;
            DynamicImage::ImageLuma8(gray_buf)
        };

        // 2. Convertir en Luma (Niveaux de gris) et appliquer le seuil
        let grayscale_img = dynamic_img.to_luma8();
        let mut binary_img = grayscale_img.clone();

        for pixel in binary_img.pixels_mut() {
            pixel.0[0] = if pixel.0[0] >= **threshold { 255 } else { 0 };
        }

        // 3. Sauvegarder le résultat sous forme de RawImage (Niveaux de gris, donc 1 canal)
        self.output = Some(Arc::new(RawImage {
            width: binary_img.width(),
            height: binary_img.height(),
            pixels: binary_img.into_raw(),
        }));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rgb_image(width: u32, height: u32, pixels: Vec<u8>) -> Arc<RawImage> {
        Arc::new(RawImage {
            width,
            height,
            pixels,
        })
    }

    fn create_test_gray_image(width: u32, height: u32, pixels: Vec<u8>) -> Arc<RawImage> {
        Arc::new(RawImage {
            width,
            height,
            pixels,
        })
    }

    #[test]
    fn test_threshold_happy_path_rgb() {
        let image = create_test_rgb_image(1, 2, vec![255, 0, 0, 0, 0, 0]);
        let mut proc = ImageThresholdProcessor::new("threshold".into());
        proc.set_input(vec![image, Arc::new(128u8)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        assert!(!output.is_empty());

        // FIX : On cible directement `RawImage` (pas de double Arc)
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 1);
        assert_eq!(result.height, 2);
    }

    #[test]
    fn test_threshold_binarization_rgb() {
        let image = create_test_rgb_image(2, 1, vec![255, 0, 0, 0, 0, 0]);
        let mut proc = ImageThresholdProcessor::new("threshold".into());
        proc.set_input(vec![image, Arc::new(100u8)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();

        // Calcul attendu de la luminance pour [255, 0, 0] : (0.299 * 255) = 76
        let expected_first = if 76 >= 100 { 255 } else { 0 };
        assert_eq!(result.pixels[0], expected_first);
        assert_eq!(result.pixels[1], 0);
    }

    #[test]
    fn test_threshold_greyscale() {
        let image = create_test_gray_image(2, 1, vec![200, 100]);
        let mut proc = ImageThresholdProcessor::new("threshold".into());
        proc.set_input(vec![image, Arc::new(150u8)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();

        assert_eq!(result.pixels[0], 255);
        assert_eq!(result.pixels[1], 0);
    }

    #[test]
    fn test_threshold_without_image_returns_error() {
        let mut proc = ImageThresholdProcessor::new("threshold".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_threshold_missing_threshold_returns_error() {
        let image = create_test_gray_image(2, 1, vec![200, 100]);
        let mut proc = ImageThresholdProcessor::new("threshold".into());
        let result = proc.set_input(vec![image]);
        assert!(matches!(
            result.unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }
}
