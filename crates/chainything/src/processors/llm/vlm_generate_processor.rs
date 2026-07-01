use std::io::Cursor;
use std::sync::Arc;

use crate::llm::LlmHandle;
use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;

/// Generates text from an image and a prompt using any vision-capable `LLM`
/// produced upstream.
///
/// This processor is provider-agnostic: it consumes an [`LlmHandle`] on its
/// third input slot, so any loader (Ollama, ...) that yields a vision-capable
/// model can be plugged in interchangeably.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>` (the image),
///   `inputs[1]` = `Arc<String>` (the prompt),
///   `inputs[2]` = `Arc<LlmHandle>` (the model to use).
/// - **Output:** one `Arc<String>` containing the generated text.
/// - **Errors:**
///   - [`ProcessorError::MissingInput`] if the image, the prompt or the model
///     is absent.
///   - [`ProcessorError::InvalidInput`] if the inputs have the wrong type.
///   - [`ProcessorError::ComputingError`] if encoding the image or the model
///     call fails.
pub struct VlmGenerateProcessor {
    id: String,
    image: Option<Arc<RawImage>>,
    prompt: Option<Arc<String>>,
    llm: Option<Arc<LlmHandle>>,
    output: Option<Arc<String>>,
}

impl VlmGenerateProcessor {
    pub fn new(id: String) -> Self {
        VlmGenerateProcessor {
            id,
            image: None,
            prompt: None,
            llm: None,
            output: None,
        }
    }

    /// Encodes a [`RawImage`] to PNG bytes so it can be sent to the backend.
    fn encode_png(image: &RawImage) -> Result<Vec<u8>, ProcessorError> {
        let expected_rgb = (image.width * image.height * 3) as usize;
        let mut buffer = Vec::new();

        let dynamic = if image.pixels.len() == expected_rgb {
            let rgb = image::RgbImage::from_raw(image.width, image.height, image.pixels.clone())
                .ok_or_else(|| {
                    ProcessorError::ComputingError("Invalid RGB image buffer".to_string())
                })?;
            image::DynamicImage::ImageRgb8(rgb)
        } else {
            let grey = image::GrayImage::from_raw(image.width, image.height, image.pixels.clone())
                .ok_or_else(|| {
                    ProcessorError::ComputingError("Invalid greyscale image buffer".to_string())
                })?;
            image::DynamicImage::ImageLuma8(grey)
        };

        dynamic
            .write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| {
                ProcessorError::ComputingError(format!("Failed to encode image: {}", e))
            })?;

        Ok(buffer)
    }
}

impl Processor for VlmGenerateProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 3 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 3 inputs (image, prompt, LLM), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let image_input = inputs.remove(0);
        let prompt_input = inputs.remove(0);
        let llm_input = inputs.remove(0);

        self.image = Some(image_input.downcast::<RawImage>().map_err(|_| {
            ProcessorError::InvalidInput(format!(
                "Invalid input type for image (expected RawImage) for processor {}",
                self.id()
            ))
        })?);

        self.prompt = Some(prompt_input.downcast::<String>().map_err(|_| {
            ProcessorError::InvalidInput(format!(
                "Invalid input type for prompt (expected String) for processor {}",
                self.id()
            ))
        })?);

        self.llm = Some(llm_input.downcast::<LlmHandle>().map_err(|_| {
            ProcessorError::InvalidInput(format!(
                "Invalid input type for model (expected LLM) for processor {}",
                self.id()
            ))
        })?);

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
        let image = self.image.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing image for processor {}", self.id()))
        })?;

        let prompt = self.prompt.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing prompt for processor {}", self.id()))
        })?;

        let llm = self.llm.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing LLM for processor {}", self.id()))
        })?;

        let png = Self::encode_png(image)?;

        let text = llm
            .generate_with_images(prompt.as_str(), std::slice::from_ref(&png))
            .map_err(|e| ProcessorError::ComputingError(e.to_string()))?;

        self.output = Some(Arc::new(text));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Llm, LlmError};

    struct EchoVlm;

    impl Llm for EchoVlm {
        fn generate(&self, prompt: &str) -> Result<String, LlmError> {
            Ok(format!("echo: {}", prompt))
        }
        fn generate_with_images(
            &self,
            prompt: &str,
            images: &[Vec<u8>],
        ) -> Result<String, LlmError> {
            Ok(format!("echo({} image(s)): {}", images.len(), prompt))
        }
        fn model(&self) -> &str {
            "echo"
        }
    }

    fn dummy_image() -> Arc<RawImage> {
        Arc::new(RawImage {
            width: 1,
            height: 1,
            pixels: vec![255, 0, 0],
        })
    }

    #[test]
    fn test_happy_path_generates_text() {
        let mut proc = VlmGenerateProcessor::new("vlm".into());
        proc.set_input(vec![
            dummy_image(),
            Arc::new("describe".to_string()),
            Arc::new(LlmHandle::new(EchoVlm)),
        ])
        .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let text = output[0].downcast_ref::<String>().unwrap();
        assert_eq!(text, "echo(1 image(s)): describe");
    }

    #[test]
    fn test_missing_inputs_returns_error() {
        let mut proc = VlmGenerateProcessor::new("vlm".into());
        assert!(matches!(
            proc.set_input(vec![dummy_image(), Arc::new("describe".to_string())])
                .unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_wrong_image_type_returns_invalid_input() {
        let mut proc = VlmGenerateProcessor::new("vlm".into());
        let inputs: Vec<Arc<dyn std::any::Any + Send + Sync>> = vec![
            Arc::new(42u32),
            Arc::new("describe".to_string()),
            Arc::new(LlmHandle::new(EchoVlm)),
        ];
        assert!(matches!(
            proc.set_input(inputs).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_wrong_llm_type_returns_invalid_input() {
        let mut proc = VlmGenerateProcessor::new("vlm".into());
        let inputs: Vec<Arc<dyn std::any::Any + Send + Sync>> = vec![
            dummy_image(),
            Arc::new("describe".to_string()),
            Arc::new(42u32),
        ];
        assert!(matches!(
            proc.set_input(inputs).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_process_without_input_returns_missing_input() {
        let mut proc = VlmGenerateProcessor::new("vlm".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_encode_png_produces_valid_signature() {
        let png = VlmGenerateProcessor::encode_png(&dummy_image()).unwrap();
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    }
}
