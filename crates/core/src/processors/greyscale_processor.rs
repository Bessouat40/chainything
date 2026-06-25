use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};

/// An RGB image with packed pixel data.
///
/// `pixels` is a flat `Vec<u8>` in row-major order, where each pixel is 3 consecutive
/// bytes `[R, G, B]`. Expected length: `width * height * 3`.
#[derive(Clone)]
pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Converts an RGB [`RawImage`] to greyscale using the luminosity formula:
/// `Y = 0.299R + 0.587G + 0.114B`.
///
/// - **Input:** one `Arc<RawImage>` with RGB pixel data.
/// - **Output:** one `Arc<RawImage>` with the same dimensions, one byte per pixel.
pub struct GreyScaleProcessor {
    id: String,
    input: Option<Arc<RawImage>>,
    output: Option<Arc<RawImage>>,
}

impl GreyScaleProcessor {
    pub fn new(id: String) -> GreyScaleProcessor {
        GreyScaleProcessor {
            id,
            input: None,
            output: None,
        }
    }
}

impl Processor for GreyScaleProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    /// - **Input:** `inputs[0]` must be an `Arc<RawImage>` with RGB pixel data.
    /// - **Errors:** [`ProcessorError::MissingInput`] if `inputs` is empty,
    ///   [`ProcessorError::InvalidInput`] if the value is not a `RawImage`.
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

    /// Returns a single `Arc<RawImage>` with one byte per pixel (greyscale).
    /// Empty if [`process`](Self::process) has not been called yet.
    fn get_output(&self) -> Vec<Arc<dyn std::any::Any + Send + Sync>> {
        self.output
            .clone()
            .into_iter()
            .map(|out| out as Arc<dyn std::any::Any + Send + Sync>)
            .collect()
    }

    /// Converts the input RGB image to greyscale.
    ///
    /// - **Errors:** [`ProcessorError::MissingInput`] if [`set_input`](Self::set_input)
    ///   was not called first.
    fn process(&mut self) -> Result<(), ProcessorError> {
        if let Some(input) = &self.input {
            let mut greyscale_pixels: Vec<u8> = Vec::with_capacity(input.pixels.len() / 3);

            for chunk in input.pixels.chunks(3) {
                if chunk.len() == 3 {
                    let value = 0.299 * (chunk[0] as f32)
                        + 0.587 * (chunk[1] as f32)
                        + 0.114 * (chunk[2] as f32);
                    greyscale_pixels.push(value as u8);
                }
            }

            self.output = Some(Arc::new(RawImage {
                width: input.width,
                height: input.height,
                pixels: greyscale_pixels,
            }));

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

    fn run(pixels: Vec<u8>) -> Vec<u8> {
        let image = Arc::new(RawImage { width: 1, height: pixels.len() as u32 / 3, pixels });
        let mut proc = GreyScaleProcessor::new("greyscale".into());
        proc.set_input(vec![image]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        output[0].downcast_ref::<RawImage>().unwrap().pixels.clone()
    }

    #[test]
    fn test_luminosity_formula() {
        // Y = 0.299 * 255 ≈ 76
        assert_eq!(run(vec![255, 0, 0]), vec![76]);
    }

    #[test]
    fn test_output_has_one_byte_per_pixel() {
        let pixels = vec![100u8, 150, 200, 100, 150, 200, 100, 150, 200, 100, 150, 200];
        assert_eq!(run(pixels).len(), 4);
    }

    #[test]
    fn test_process_without_input_returns_error() {
        let mut proc = GreyScaleProcessor::new("greyscale".into());
        assert!(matches!(proc.process().unwrap_err(), ProcessorError::MissingInput(_)));
    }

    #[test]
    fn test_set_input_wrong_type_returns_error() {
        let mut proc = GreyScaleProcessor::new("greyscale".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42u32);
        assert!(matches!(proc.set_input(vec![bad]).unwrap_err(), ProcessorError::InvalidInput(_)));
    }

    #[test]
    fn test_set_input_empty_returns_error() {
        let mut proc = GreyScaleProcessor::new("greyscale".into());
        assert!(matches!(proc.set_input(vec![]).unwrap_err(), ProcessorError::MissingInput(_)));
    }
}