use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::model3d::mesh::Mesh3D;
use std::sync::Arc;

/// The `ModelReaderProcessor` loads a 3D model file from the filesystem.
///
/// ### Input
/// * Expects a single `Arc<String>` representing the path to a Wavefront OBJ
///   file.
///
/// ### Output
/// * Produces an `Arc<Mesh3D>` containing the parsed vertices and faces.
///
/// ### Errors
/// * [`ProcessorError::MissingInput`] if no path is provided.
/// * [`ProcessorError::InvalidInput`] if the input is not a `String`.
/// * [`ProcessorError::ComputingError`] if the file cannot be read or parsed.
pub struct ModelReaderProcessor {
    id: String,
    input: Option<Arc<String>>,
    output: Option<Arc<Mesh3D>>,
}

impl ModelReaderProcessor {
    pub fn new(id: String) -> ModelReaderProcessor {
        ModelReaderProcessor {
            id,
            input: None,
            output: None,
        }
    }
}

impl Processor for ModelReaderProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.is_empty() {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 1 input (path), got 0",
                self.id()
            )));
        }

        let first_input = inputs.remove(0);

        if let Ok(typed_input) = first_input.downcast::<String>() {
            self.input = Some(typed_input);
            Ok(())
        } else {
            Err(ProcessorError::InvalidInput(format!(
                "Invalid input type (expected String) for processor {}",
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
        let input = self.input.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing input for processor {}", self.id()))
        })?;

        let content = std::fs::read_to_string(input.as_ref()).map_err(|e| {
            ProcessorError::ComputingError(format!("Could not read model '{}': {}", input, e))
        })?;

        let mesh = Mesh3D::from_obj(&content)?;
        self.output = Some(Arc::new(mesh));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_missing_input_fails() {
        let mut processor = ModelReaderProcessor::new("test".to_string());
        assert!(matches!(
            processor.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_invalid_input_type_fails() {
        let mut processor = ModelReaderProcessor::new("test".to_string());
        let inputs: Vec<Arc<dyn std::any::Any + Send + Sync>> = vec![Arc::new(123)];
        assert!(matches!(
            processor.set_input(inputs).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_file_not_found_fails() {
        let mut processor = ModelReaderProcessor::new("test".to_string());
        let inputs: Vec<Arc<dyn std::any::Any + Send + Sync>> =
            vec![Arc::new("non_existent.obj".to_string())];
        processor.set_input(inputs).unwrap();
        assert!(matches!(
            processor.process().unwrap_err(),
            ProcessorError::ComputingError(_)
        ));
    }

    #[test]
    fn test_reads_obj_from_disk() {
        let dir = std::env::temp_dir();
        let path = dir.join("chainything_reader_test.obj");
        std::fs::write(&path, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();

        let mut processor = ModelReaderProcessor::new("test".to_string());
        processor
            .set_input(vec![Arc::new(path.to_string_lossy().to_string())])
            .unwrap();
        processor.process().unwrap();

        let output = processor.get_output();
        let mesh = output[0].downcast_ref::<Mesh3D>().unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.face_count(), 1);

        let _ = std::fs::remove_file(&path);
    }
}
