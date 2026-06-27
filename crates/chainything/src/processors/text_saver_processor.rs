use std::{any::Any, sync::Arc};

use crate::processors::base_processor::{Processor, ProcessorError};

/// Saves a text `String` to a file on the filesystem.
///
/// Useful as a sink for text-producing processors such as
/// [`OllamaProcessor`](crate::processors::ollama_processor::OllamaProcessor).
///
/// - **Input:** `inputs[0]` = `Arc<String>` (the text content),
///   `inputs[1]` = `Arc<String>` (the destination path).
/// - **Output:** none.
/// - **Errors:**
///   - [`ProcessorError::MissingInput`] if either input is absent.
///   - [`ProcessorError::InvalidInput`] if the types are mismatched.
///   - [`ProcessorError::ComputingError`] if the file cannot be written.
pub struct TextSaveProcessor {
    id: String,
    input: Option<Arc<String>>,
    output_path: Option<Arc<String>>,
}

impl TextSaveProcessor {
    pub fn new(id: String) -> Self {
        TextSaveProcessor {
            id,
            input: None,
            output_path: None,
        }
    }
}

impl Processor for TextSaveProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (text, path), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let first = inputs.remove(0);
        let second = inputs.remove(0);

        self.input = Some(first.downcast::<String>().map_err(|_| {
            ProcessorError::InvalidInput("First input must be a String (text)".to_string())
        })?);

        self.output_path = Some(second.downcast::<String>().map_err(|_| {
            ProcessorError::InvalidInput("Second input must be a String (path)".to_string())
        })?);

        Ok(())
    }

    fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        Vec::new()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let input = self
            .input
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput("Missing text input".to_string()))?;

        let output_path = self
            .output_path
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput("Missing output path".to_string()))?;

        std::fs::write(output_path.as_str(), input.as_bytes())
            .map_err(|e| ProcessorError::ComputingError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_inputs_fails() {
        let mut processor = TextSaveProcessor::new("text_save".to_string());
        assert!(matches!(
            processor.set_input(vec![]).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_invalid_input_types_fails() {
        let mut processor = TextSaveProcessor::new("text_save".to_string());
        let inputs: Vec<Arc<dyn Any + Send + Sync>> =
            vec![Arc::new(10u32), Arc::new("path".to_string())];
        assert!(matches!(
            processor.set_input(inputs).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_process_without_input_fails() {
        let mut processor = TextSaveProcessor::new("text_save".to_string());
        assert!(matches!(
            processor.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_happy_path_writes_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("chainything_text_save_test.txt");
        let path_str = path.to_string_lossy().to_string();

        let mut processor = TextSaveProcessor::new("text_save".to_string());
        processor
            .set_input(vec![
                Arc::new("hello world".to_string()),
                Arc::new(path_str.clone()),
            ])
            .unwrap();
        processor.process().unwrap();

        let written = std::fs::read_to_string(&path_str).unwrap();
        assert_eq!(written, "hello world");

        let _ = std::fs::remove_file(&path_str);
    }
}
