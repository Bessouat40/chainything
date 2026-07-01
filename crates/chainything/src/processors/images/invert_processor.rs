use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;

/// Inverts the colours of a [`RawImage`], producing a photographic negative.
///
/// Each channel byte `p` is replaced by `255 - p`. Works transparently on both
/// RGB (3 bytes per pixel) and greyscale (1 byte per pixel) images since the
/// operation is applied per byte.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>`.
/// - **Output:** one `Arc<RawImage>` with the same dimensions and channel layout,
///   inverted.
/// - **Errors:** [`ProcessorError::MissingInput`] if no input is provided,
///   [`ProcessorError::InvalidInput`] if the value is not a `RawImage`.
pub struct InvertProcessor {
    id: String,
    input: Option<Arc<RawImage>>,
    output: Option<Arc<RawImage>>,
}

impl InvertProcessor {
    pub fn new(id: String) -> Self {
        InvertProcessor {
            id,
            input: None,
            output: None,
        }
    }
}

impl Processor for InvertProcessor {
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
            let inverted_pixels: Vec<u8> = input.pixels.iter().map(|p| 255 - p).collect();

            self.output = Some(Arc::new(RawImage {
                width: input.width,
                height: input.height,
                pixels: inverted_pixels,
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

    fn run(width: u32, height: u32, pixels: Vec<u8>) -> Vec<u8> {
        let image = Arc::new(RawImage {
            width,
            height,
            pixels,
        });
        let mut proc = InvertProcessor::new("invert".into());
        proc.set_input(vec![image]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        output[0].downcast_ref::<RawImage>().unwrap().pixels.clone()
    }

    #[test]
    fn test_invert_rgb() {
        assert_eq!(run(1, 1, vec![0, 128, 255]), vec![255, 127, 0]);
    }

    #[test]
    fn test_invert_greyscale() {
        assert_eq!(run(2, 1, vec![10, 245]), vec![245, 10]);
    }

    #[test]
    fn test_invert_is_involutive() {
        let original = vec![3u8, 17, 200, 255];
        let once = run(4, 1, original.clone());
        let twice = run(4, 1, once);
        assert_eq!(twice, original);
    }

    #[test]
    fn test_process_without_input_returns_error() {
        let mut proc = InvertProcessor::new("invert".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_set_input_wrong_type_returns_error() {
        let mut proc = InvertProcessor::new("invert".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42u32);
        assert!(matches!(
            proc.set_input(vec![bad]).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_set_input_empty_returns_error() {
        let mut proc = InvertProcessor::new("invert".into());
        assert!(matches!(
            proc.set_input(vec![]).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }
}
