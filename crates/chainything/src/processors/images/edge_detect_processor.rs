use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;

/// Detects edges in a [`RawImage`] using the Sobel operator.
///
/// The image is first converted to greyscale (using the luminosity formula for
/// RGB inputs), then the horizontal and vertical Sobel kernels are convolved
/// over it. The gradient magnitude `sqrt(gx^2 + gy^2)` is clamped to `0..=255`
/// and returned as a single-channel (greyscale) image where bright pixels mark
/// strong edges.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>` (RGB or greyscale).
/// - **Output:** one `Arc<RawImage>` (greyscale) of the same dimensions.
/// - **Errors:** [`ProcessorError::MissingInput`] if no input is provided,
///   [`ProcessorError::InvalidInput`] if the value is not a `RawImage`.
pub struct EdgeDetectProcessor {
    id: String,
    input: Option<Arc<RawImage>>,
    output: Option<Arc<RawImage>>,
}

impl EdgeDetectProcessor {
    pub fn new(id: String) -> Self {
        EdgeDetectProcessor {
            id,
            input: None,
            output: None,
        }
    }

    /// Converts the input to a flat greyscale buffer (one byte per pixel).
    fn to_grey(image: &RawImage) -> Vec<u8> {
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;
        if is_rgb {
            image
                .pixels
                .chunks_exact(3)
                .map(|c| (0.299 * c[0] as f32 + 0.587 * c[1] as f32 + 0.114 * c[2] as f32) as u8)
                .collect()
        } else {
            image.pixels.clone()
        }
    }

    fn sobel(image: &RawImage) -> RawImage {
        let width = image.width as usize;
        let height = image.height as usize;
        let grey = Self::to_grey(image);

        // Images too small to have an interior return an all-black result.
        if width < 3 || height < 3 {
            return RawImage {
                width: image.width,
                height: image.height,
                pixels: vec![0u8; width * height],
            };
        }

        const GX: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
        const GY: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

        let mut out = vec![0u8; width * height];

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let mut sum_x = 0i32;
                let mut sum_y = 0i32;

                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = grey[(y + ky - 1) * width + (x + kx - 1)] as i32;
                        sum_x += GX[ky][kx] * px;
                        sum_y += GY[ky][kx] * px;
                    }
                }

                let magnitude = ((sum_x * sum_x + sum_y * sum_y) as f32).sqrt();
                out[y * width + x] = magnitude.min(255.0) as u8;
            }
        }

        RawImage {
            width: image.width,
            height: image.height,
            pixels: out,
        }
    }
}

impl Processor for EdgeDetectProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.is_empty() {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 1 input, got 0",
                self.id()
            )));
        }

        let first_input = inputs.remove(0);

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input = Some(typed_image);
            Ok(())
        } else {
            Err(ProcessorError::InvalidInput(format!(
                "Invalid input type (expected RawImage) for processor {}",
                self.id()
            )))
        }
    }

    fn get_output(&self) -> Vec<Arc<dyn std::any::Any + Send + Sync>> {
        self.output
            .clone()
            .into_iter()
            .map(|out| out as Arc<dyn std::any::Any + Send + Sync>)
            .collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        if let Some(input) = &self.input {
            self.output = Some(Arc::new(Self::sobel(input)));
            Ok(())
        } else {
            Err(ProcessorError::MissingInput(format!(
                "Missing input for processor {}",
                self.id()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_output_is_single_channel() {
        let image = Arc::new(RawImage {
            width: 4,
            height: 4,
            pixels: vec![128u8; 4 * 4 * 3],
        });
        let mut proc = EdgeDetectProcessor::new("edge".into());
        proc.set_input(vec![image]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 4);
        assert_eq!(result.pixels.len(), 16);
    }

    #[test]
    fn test_flat_image_has_no_edges() {
        // A uniform image has zero gradient everywhere.
        let image = Arc::new(RawImage {
            width: 5,
            height: 5,
            pixels: vec![200u8; 5 * 5],
        });
        let mut proc = EdgeDetectProcessor::new("edge".into());
        proc.set_input(vec![image]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert!(result.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn test_vertical_edge_is_detected() {
        // Left half black, right half white -> a strong vertical edge.
        let width = 5usize;
        let height = 5usize;
        let mut pixels = vec![0u8; width * height];
        for y in 0..height {
            for x in 0..width {
                if x >= width / 2 {
                    pixels[y * width + x] = 255;
                }
            }
        }
        let image = Arc::new(RawImage {
            width: width as u32,
            height: height as u32,
            pixels,
        });
        let mut proc = EdgeDetectProcessor::new("edge".into());
        proc.set_input(vec![image]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        // At least one interior pixel should register a strong edge.
        assert!(result.pixels.iter().any(|&p| p > 100));
    }

    #[test]
    fn test_process_without_input_returns_error() {
        let mut proc = EdgeDetectProcessor::new("edge".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_set_input_wrong_type_returns_error() {
        let mut proc = EdgeDetectProcessor::new("edge".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42u32);
        assert!(matches!(
            proc.set_input(vec![bad]).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }
}
