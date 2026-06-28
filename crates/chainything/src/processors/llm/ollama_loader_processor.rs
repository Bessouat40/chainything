use std::sync::Arc;

use crate::llm::{LlmHandle, ollama::OllamaLlm};
use crate::processors::base_processor::{Processor, ProcessorError};

/// Default Ollama model used when none is provided.
const DEFAULT_MODEL: &str = "llama3.2";

/// Builds an [`LlmHandle`] backed by a local Ollama model.
///
/// This is a *source* processor: it takes no upstream data, only a model name
/// as a parameter, and outputs a reusable `LLM` handle. Connect its output to
/// any processor that consumes an `LLM` (e.g. `LLMGenerate`).
///
/// The server URL is read from the `OLLAMA_HOST` environment variable, falling
/// back to `http://localhost:11434`.
///
/// - **Input:** `inputs[0]` (optional) = `Arc<String>` (the model name, e.g.
///   `"llama3.2"`). Defaults to `llama3.2` when missing or empty.
/// - **Output:** one `Arc<LlmHandle>`.
pub struct OllamaLoaderProcessor {
    id: String,
    model: Option<Arc<String>>,
    output: Option<Arc<LlmHandle>>,
}

impl OllamaLoaderProcessor {
    pub fn new(id: String) -> Self {
        OllamaLoaderProcessor {
            id,
            model: None,
            output: None,
        }
    }
}

impl Processor for OllamaLoaderProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        // The model name is an optional parameter; an empty value uses the default.
        if !inputs.is_empty() {
            let model_input = inputs.remove(0);
            if let Ok(model) = model_input.downcast::<String>()
                && !model.trim().is_empty()
            {
                self.model = Some(model);
            }
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
        let model = self
            .model
            .as_ref()
            .map(|m| m.as_str())
            .unwrap_or(DEFAULT_MODEL);

        self.output = Some(Arc::new(LlmHandle::new(OllamaLlm::new(model))));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outputs_handle_with_given_model() {
        let mut proc = OllamaLoaderProcessor::new("loader".into());
        proc.set_input(vec![Arc::new("mistral".to_string())])
            .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let handle = output[0].downcast_ref::<LlmHandle>().unwrap();
        assert_eq!(handle.model(), "mistral");
    }

    #[test]
    fn test_defaults_to_llama_when_no_input() {
        let mut proc = OllamaLoaderProcessor::new("loader".into());
        proc.set_input(vec![]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let handle = output[0].downcast_ref::<LlmHandle>().unwrap();
        assert_eq!(handle.model(), DEFAULT_MODEL);
    }

    #[test]
    fn test_blank_model_uses_default() {
        let mut proc = OllamaLoaderProcessor::new("loader".into());
        proc.set_input(vec![Arc::new("   ".to_string())]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let handle = output[0].downcast_ref::<LlmHandle>().unwrap();
        assert_eq!(handle.model(), DEFAULT_MODEL);
    }
}
