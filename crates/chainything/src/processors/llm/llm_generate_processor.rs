use std::sync::Arc;

use crate::llm::LlmHandle;
use crate::processors::base_processor::{Processor, ProcessorError};

/// Generates text from a prompt using any `LLM` produced upstream.
///
/// This processor is provider-agnostic: it consumes an [`LlmHandle`] on its
/// second input slot, so any loader (Ollama, OpenAI, Mistral, Claude, ...) can
/// be plugged in interchangeably.
///
/// - **Input:** `inputs[0]` = `Arc<String>` (the prompt),
///   `inputs[1]` = `Arc<LlmHandle>` (the model to use).
/// - **Output:** one `Arc<String>` containing the generated text.
/// - **Errors:**
///   - [`ProcessorError::MissingInput`] if the prompt or the model is absent.
///   - [`ProcessorError::InvalidInput`] if the inputs have the wrong type.
///   - [`ProcessorError::ComputingError`] if the model call fails.
pub struct LlmGenerateProcessor {
    id: String,
    prompt: Option<Arc<String>>,
    llm: Option<Arc<LlmHandle>>,
    output: Option<Arc<String>>,
}

impl LlmGenerateProcessor {
    pub fn new(id: String) -> Self {
        LlmGenerateProcessor {
            id,
            prompt: None,
            llm: None,
            output: None,
        }
    }
}

impl Processor for LlmGenerateProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (prompt, LLM), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let prompt_input = inputs.remove(0);
        let llm_input = inputs.remove(0);

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
        let prompt = self.prompt.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing prompt for processor {}", self.id()))
        })?;

        let llm = self.llm.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing LLM for processor {}", self.id()))
        })?;

        let text = llm
            .generate(prompt.as_str())
            .map_err(|e| ProcessorError::ComputingError(e.to_string()))?;

        self.output = Some(Arc::new(text));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Llm, LlmError};

    struct EchoLlm;

    impl Llm for EchoLlm {
        fn generate(&self, prompt: &str) -> Result<String, LlmError> {
            Ok(format!("echo: {}", prompt))
        }
        fn model(&self) -> &str {
            "echo"
        }
    }

    #[test]
    fn test_happy_path_generates_text() {
        let mut proc = LlmGenerateProcessor::new("gen".into());
        proc.set_input(vec![
            Arc::new("hello".to_string()),
            Arc::new(LlmHandle::new(EchoLlm)),
        ])
        .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let text = output[0].downcast_ref::<String>().unwrap();
        assert_eq!(text, "echo: hello");
    }

    #[test]
    fn test_missing_inputs_returns_error() {
        let mut proc = LlmGenerateProcessor::new("gen".into());
        assert!(matches!(
            proc.set_input(vec![Arc::new("hello".to_string())])
                .unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_wrong_llm_type_returns_invalid_input() {
        let mut proc = LlmGenerateProcessor::new("gen".into());
        let inputs: Vec<Arc<dyn std::any::Any + Send + Sync>> =
            vec![Arc::new("hello".to_string()), Arc::new(42u32)];
        assert!(matches!(
            proc.set_input(inputs).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_process_without_input_returns_missing_input() {
        let mut proc = LlmGenerateProcessor::new("gen".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }
}
