use std::time::Duration;

use super::{Llm, LlmError};

/// Default Ollama server endpoint.
const DEFAULT_HOST: &str = "http://localhost:11434";
/// Maximum time to wait for a generation to complete (generation can be slow).
const REQUEST_TIMEOUT_SECS: u64 = 300;

/// [`Llm`] implementation backed by a local [Ollama](https://ollama.com) server.
///
/// The server URL defaults to `http://localhost:11434` and can be overridden
/// with the `OLLAMA_HOST` environment variable (or [`OllamaLlm::with_host`]).
/// The target model must have been pulled beforehand (e.g. `ollama pull llama3.2`).
pub struct OllamaLlm {
    model: String,
    host: String,
}

impl OllamaLlm {
    /// Creates a handle for `model`, reading the host from `OLLAMA_HOST`
    /// (falling back to `http://localhost:11434`).
    pub fn new(model: impl Into<String>) -> Self {
        let host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        Self {
            model: model.into(),
            host,
        }
    }

    /// Creates a handle for `model` targeting an explicit `host`.
    pub fn with_host(model: impl Into<String>, host: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            host: host.into(),
        }
    }
}

impl Llm for OllamaLlm {
    fn generate(&self, prompt: &str) -> Result<String, LlmError> {
        let url = format!("{}/api/generate", self.host.trim_end_matches('/'));

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build();

        let response: serde_json::Value = agent
            .post(&url)
            .send_json(serde_json::json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false,
            }))
            .map_err(|e| LlmError::Request(format!("Ollama request to {} failed: {}", url, e)))?
            .into_json()
            .map_err(|e| LlmError::Response(format!("Failed to parse Ollama response: {}", e)))?;

        response
            .get("response")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                LlmError::Response("Ollama response did not contain a 'response' field".to_string())
            })
    }

    fn model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uses_default_host_when_env_absent() {
        // Avoid depending on the environment of the test runner.
        unsafe {
            std::env::remove_var("OLLAMA_HOST");
        }
        let llm = OllamaLlm::new("llama3.2");
        assert_eq!(llm.model(), "llama3.2");
        assert_eq!(llm.host, DEFAULT_HOST);
    }

    #[test]
    fn test_with_host_overrides_endpoint() {
        let llm = OllamaLlm::with_host("mistral", "http://192.168.1.10:11434");
        assert_eq!(llm.model(), "mistral");
        assert_eq!(llm.host, "http://192.168.1.10:11434");
    }
}
