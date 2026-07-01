use image::{DynamicImage, RgbImage, imageops::FilterType};
use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;

/// Blends two images together using a linear (alpha) blend.
///
/// The two inputs are the *base* image and the *overlay* image. Each is
/// converted to RGB; if their dimensions differ, the overlay is resized to match
/// the base. The result is computed per channel as
/// `out = base * (1 - a) + overlay * a`, where `a = alpha / 100`.
///
/// This is useful for compositing workflows: extract a subject, transform it,
/// then merge it back over a background.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>` (base), `inputs[1]` = `Arc<RawImage>`
///   (overlay), `inputs[2]` = `Arc<u32>` / `Arc<String>` (alpha, a percentage in
///   `0..=100`).
/// - **Output:** one `Arc<RawImage>` (RGB) with the base image's dimensions.
/// - **Errors:** [`ProcessorError::MissingInput`] if fewer than 3 inputs,
///   [`ProcessorError::InvalidInput`] if a type is wrong,
///   [`ProcessorError::ComputingError`] if a pixel buffer is malformed.
pub struct MergeProcessor {
    id: String,
    base: Option<Arc<RawImage>>,
    overlay: Option<Arc<RawImage>>,
    alpha: Option<u32>,
    output: Option<Arc<RawImage>>,
}

impl MergeProcessor {
    pub fn new(id: String) -> Self {
        MergeProcessor {
            id,
            base: None,
            overlay: None,
            alpha: None,
            output: None,
        }
    }

    /// Rebuilds an RGB buffer from a [`RawImage`], expanding greyscale to RGB.
    fn to_rgb(image: &RawImage) -> Result<RgbImage, ProcessorError> {
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;
        if is_rgb {
            RgbImage::from_raw(image.width, image.height, image.pixels.clone())
                .ok_or_else(|| ProcessorError::ComputingError("Invalid RGB buffer".into()))
        } else {
            let gray = image::GrayImage::from_raw(image.width, image.height, image.pixels.clone())
                .ok_or_else(|| ProcessorError::ComputingError("Invalid Grayscale buffer".into()))?;
            Ok(DynamicImage::ImageLuma8(gray).to_rgb8())
        }
    }
}

impl Processor for MergeProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 3 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 3 inputs (base, overlay, alpha), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let base_input = inputs.remove(0);
        let overlay_input = inputs.remove(0);
        let alpha_input = inputs.remove(0);

        if let Ok(typed_image) = base_input.downcast::<RawImage>() {
            self.base = Some(typed_image);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for base image (expected RawImage) for processor {}",
                self.id()
            )));
        }

        if let Ok(typed_image) = overlay_input.downcast::<RawImage>() {
            self.overlay = Some(typed_image);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for overlay image (expected RawImage) for processor {}",
                self.id()
            )));
        }

        let alpha_val = if let Ok(typed_alpha) = alpha_input.clone().downcast::<u32>() {
            *typed_alpha
        } else if let Ok(typed_string) = alpha_input.downcast::<String>() {
            typed_string.parse().map_err(|_| {
                ProcessorError::InvalidInput(format!(
                    "Cannot parse alpha as u32 for processor {}",
                    self.id()
                ))
            })?
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for alpha (expected u32 or String) for processor {}",
                self.id()
            )));
        };

        self.alpha = Some(alpha_val.min(100));
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
        let base = self.base.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing base image for processor {}", self.id()))
        })?;

        let overlay = self.overlay.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!(
                "Missing overlay image for processor {}",
                self.id()
            ))
        })?;

        let alpha = self.alpha.ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing alpha for processor {}", self.id()))
        })?;

        let base_rgb = Self::to_rgb(base)?;
        let mut overlay_rgb = Self::to_rgb(overlay)?;

        // Align the overlay to the base's dimensions so the blend is well-defined.
        if overlay_rgb.width() != base_rgb.width() || overlay_rgb.height() != base_rgb.height() {
            overlay_rgb = image::imageops::resize(
                &overlay_rgb,
                base_rgb.width(),
                base_rgb.height(),
                FilterType::Triangle,
            );
        }

        let a = alpha as f32 / 100.0;
        let base_raw = base_rgb.into_raw();
        let overlay_raw = overlay_rgb.into_raw();

        let blended: Vec<u8> = base_raw
            .iter()
            .zip(overlay_raw.iter())
            .map(|(&b, &o)| {
                (b as f32 * (1.0 - a) + o as f32 * a)
                    .round()
                    .clamp(0.0, 255.0) as u8
            })
            .collect();

        self.output = Some(Arc::new(RawImage {
            width: base.width,
            height: base.height,
            pixels: blended,
        }));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb_image(width: u32, height: u32, color: [u8; 3]) -> Arc<RawImage> {
        let mut pixels = Vec::with_capacity((width * height * 3) as usize);
        for _ in 0..(width * height) {
            pixels.extend_from_slice(&color);
        }
        Arc::new(RawImage {
            width,
            height,
            pixels,
        })
    }

    fn run(
        base: Arc<RawImage>,
        overlay: Arc<RawImage>,
        alpha: Arc<dyn std::any::Any + Send + Sync>,
    ) -> RawImage {
        let mut proc = MergeProcessor::new("merge".into());
        proc.set_input(vec![base, overlay, alpha]).unwrap();
        proc.process().unwrap();
        let output = proc.get_output();
        output[0].downcast_ref::<RawImage>().unwrap().clone()
    }

    #[test]
    fn test_alpha_zero_keeps_base() {
        let base = rgb_image(2, 2, [0, 0, 0]);
        let overlay = rgb_image(2, 2, [255, 255, 255]);
        let result = run(base, overlay, Arc::new(0u32));
        assert!(result.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn test_alpha_full_keeps_overlay() {
        let base = rgb_image(2, 2, [0, 0, 0]);
        let overlay = rgb_image(2, 2, [255, 255, 255]);
        let result = run(base, overlay, Arc::new(100u32));
        assert!(result.pixels.iter().all(|&p| p == 255));
    }

    #[test]
    fn test_alpha_half_is_average() {
        let base = rgb_image(1, 1, [0, 0, 0]);
        let overlay = rgb_image(1, 1, [200, 100, 50]);
        let result = run(base, overlay, Arc::new(50u32));
        assert_eq!(result.pixels, vec![100, 50, 25]);
    }

    #[test]
    fn test_mismatched_dimensions_resizes_overlay() {
        let base = rgb_image(4, 4, [10, 20, 30]);
        let overlay = rgb_image(2, 2, [10, 20, 30]);
        let result = run(base, overlay, Arc::new(100u32));
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);
        assert_eq!(result.pixels.len(), 4 * 4 * 3);
    }

    #[test]
    fn test_greyscale_overlay_is_expanded() {
        let base = rgb_image(1, 1, [0, 0, 0]);
        let overlay = Arc::new(RawImage {
            width: 1,
            height: 1,
            pixels: vec![128],
        });
        let result = run(base, overlay, Arc::new(100u32));
        assert_eq!(result.pixels, vec![128, 128, 128]);
    }

    #[test]
    fn test_alpha_from_string() {
        let base = rgb_image(1, 1, [0, 0, 0]);
        let overlay = rgb_image(1, 1, [100, 100, 100]);
        let result = run(base, overlay, Arc::new("100".to_string()));
        assert_eq!(result.pixels, vec![100, 100, 100]);
    }

    #[test]
    fn test_missing_inputs_returns_error() {
        let base = rgb_image(1, 1, [0, 0, 0]);
        let mut proc = MergeProcessor::new("merge".into());
        assert!(matches!(
            proc.set_input(vec![base]).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_process_without_input_returns_error() {
        let mut proc = MergeProcessor::new("merge".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_wrong_base_type_returns_error() {
        let overlay = rgb_image(1, 1, [0, 0, 0]);
        let mut proc = MergeProcessor::new("merge".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new(1u32);
        assert!(matches!(
            proc.set_input(vec![bad, overlay, Arc::new(50u32)])
                .unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }
}
