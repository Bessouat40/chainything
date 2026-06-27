use std::sync::Arc;

pub mod ollama;

/// Errors that can occur while interacting with an [`Llm`] backend.
#[derive(Debug)]
pub enum LlmError {
    /// The request to the backend could not be performed (network, connection, ...).
    Request(String),
    /// The backend replied but the response could not be understood.
    Response(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Request(msg) => write!(f, "LLM request error: {}", msg),
            LlmError::Response(msg) => write!(f, "LLM response error: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

/// A text-generation model.
///
/// Implement this trait once per provider (Ollama, OpenAI, Mistral, Claude, ...).
/// Each implementation owns whatever configuration it needs (model name,
/// endpoint, API key, ...) and exposes a single [`generate`](Llm::generate)
/// entry point.
///
/// Implementations are wrapped in an [`LlmHandle`] so they can be passed between
/// processors as a uniform, cloneable value.
pub trait Llm: Send + Sync {
    /// Generates a completion for the given `prompt`.
    fn generate(&self, prompt: &str) -> Result<String, LlmError>;

    /// Human-readable identifier of the underlying model (used for logging/UI).
    fn model(&self) -> &str;
}

/// Cloneable, type-erased handle to an [`Llm`] implementation.
///
/// This is the concrete type carried on `LLM` pins between processors: a
/// provider-specific *loader* processor produces an `LlmHandle`, and any
/// processor that needs to call a model (e.g. text generation) consumes one.
/// Cloning is cheap — it only clones the inner [`Arc`].
#[derive(Clone)]
pub struct LlmHandle {
    inner: Arc<dyn Llm>,
}

impl LlmHandle {
    /// Wraps a concrete [`Llm`] implementation into a shareable handle.
    pub fn new(llm: impl Llm + 'static) -> Self {
        Self {
            inner: Arc::new(llm),
        }
    }

    /// Generates a completion for `prompt` using the wrapped model.
    pub fn generate(&self, prompt: &str) -> Result<String, LlmError> {
        self.inner.generate(prompt)
    }

    /// Returns the wrapped model's identifier.
    pub fn model(&self) -> &str {
        self.inner.model()
    }
}

impl std::fmt::Debug for LlmHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmHandle")
            .field("model", &self.model())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_handle_delegates_to_inner() {
        let handle = LlmHandle::new(EchoLlm);
        assert_eq!(handle.model(), "echo");
        assert_eq!(handle.generate("hi").unwrap(), "echo: hi");
    }

    #[test]
    fn test_handle_is_clone_and_shares_inner() {
        let handle = LlmHandle::new(EchoLlm);
        let cloned = handle.clone();
        assert_eq!(cloned.generate("x").unwrap(), "echo: x");
    }
}
