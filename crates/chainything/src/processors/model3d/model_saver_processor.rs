use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::model3d::mesh::Mesh3D;
use std::{any::Any, sync::Arc};

/// The `ModelSaveProcessor` writes a [`Mesh3D`] to disk as a Wavefront OBJ file.
///
/// ### Input
/// 1. `Arc<Mesh3D>` — the mesh to save.
/// 2. `Arc<String>` — the destination file path.
///
/// ### Output
/// * Returns an empty vector (no data output).
///
/// ### Errors
/// * [`ProcessorError::MissingInput`] if fewer than two inputs are provided.
/// * [`ProcessorError::InvalidInput`] if the input types are mismatched.
/// * [`ProcessorError::ComputingError`] if the file cannot be written.
pub struct ModelSaveProcessor {
    id: String,
    input: Option<Arc<Mesh3D>>,
    output_path: Option<Arc<String>>,
}

impl ModelSaveProcessor {
    pub fn new(id: String) -> Self {
        ModelSaveProcessor {
            id,
            input: None,
            output_path: None,
        }
    }
}

impl Processor for ModelSaveProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (Mesh3D, path), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let first = inputs.remove(0);
        let second = inputs.remove(0);

        self.input =
            Some(first.downcast::<Mesh3D>().map_err(|_| {
                ProcessorError::InvalidInput("First input must be Mesh3D".to_string())
            })?);

        self.output_path = Some(second.downcast::<String>().map_err(|_| {
            ProcessorError::InvalidInput("Second input must be String".to_string())
        })?);

        Ok(())
    }

    fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        Vec::new()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let mesh = self
            .input
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput("Missing mesh input".to_string()))?;

        let output_path = self
            .output_path
            .as_ref()
            .ok_or_else(|| ProcessorError::MissingInput("Missing output path".to_string()))?;

        std::fs::write(output_path.as_str(), mesh.to_obj())
            .map_err(|e| ProcessorError::ComputingError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mesh() -> Arc<Mesh3D> {
        Arc::new(Mesh3D::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        ))
    }

    #[test]
    fn test_missing_inputs_fails() {
        let mut processor = ModelSaveProcessor::new("save_test".to_string());
        assert!(matches!(
            processor.set_input(vec![]).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_invalid_input_types_fails() {
        let mut processor = ModelSaveProcessor::new("save_test".to_string());
        let inputs: Vec<Arc<dyn Any + Send + Sync>> =
            vec![Arc::new("path".to_string()), Arc::new(10)];
        assert!(matches!(
            processor.set_input(inputs).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_process_without_input_fails() {
        let mut processor = ModelSaveProcessor::new("save_test".to_string());
        assert!(matches!(
            processor.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_saves_obj_to_disk() {
        let dir = std::env::temp_dir();
        let path = dir.join("chainything_saver_test.obj");

        let mut processor = ModelSaveProcessor::new("save_test".to_string());
        processor
            .set_input(vec![
                sample_mesh(),
                Arc::new(path.to_string_lossy().to_string()),
            ])
            .unwrap();
        processor.process().unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        let reparsed = Mesh3D::from_obj(&written).unwrap();
        assert_eq!(reparsed.vertex_count(), 3);
        assert_eq!(reparsed.face_count(), 1);

        let _ = std::fs::remove_file(&path);
    }
}
