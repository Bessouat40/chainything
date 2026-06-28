use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::model3d::mesh::Mesh3D;
use std::sync::Arc;

/// Uniformly scales a [`Mesh3D`] about the model-space origin.
///
/// Every vertex coordinate is multiplied by the same factor, leaving the face
/// connectivity untouched.
///
/// ### Input
/// 1. `Arc<Mesh3D>` — the mesh to scale.
/// 2. The scale factor, accepted as `Arc<f32>`, `Arc<f64>`, `Arc<u32>` or
///    `Arc<String>` (parsed as `f32`).
///
/// ### Output
/// * One `Arc<Mesh3D>` with scaled vertices.
///
/// ### Errors
/// * [`ProcessorError::MissingInput`] if fewer than two inputs are provided.
/// * [`ProcessorError::InvalidInput`] if either input has an unexpected type.
pub struct ModelScaleProcessor {
    id: String,
    input: Option<Arc<Mesh3D>>,
    factor: Option<f32>,
    output: Option<Arc<Mesh3D>>,
}

impl ModelScaleProcessor {
    pub fn new(id: String) -> Self {
        ModelScaleProcessor {
            id,
            input: None,
            factor: None,
            output: None,
        }
    }
}

impl Processor for ModelScaleProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (mesh, factor), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let mesh_input = inputs.remove(0);
        let factor_input = inputs.remove(0);

        self.input = Some(mesh_input.downcast::<Mesh3D>().map_err(|_| {
            ProcessorError::InvalidInput(format!(
                "Invalid input type (expected Mesh3D) for processor {}",
                self.id()
            ))
        })?);

        if let Ok(f) = factor_input.clone().downcast::<f32>() {
            self.factor = Some(*f);
        } else if let Ok(f) = factor_input.clone().downcast::<f64>() {
            self.factor = Some(*f as f32);
        } else if let Ok(u) = factor_input.clone().downcast::<u32>() {
            self.factor = Some(*u as f32);
        } else if let Ok(s) = factor_input.downcast::<String>() {
            let parsed: f32 = s.trim().parse().map_err(|_| {
                ProcessorError::InvalidInput(format!(
                    "Cannot parse scale factor as f32 for processor {}",
                    self.id()
                ))
            })?;
            self.factor = Some(parsed);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for factor (expected f32, f64, u32 or String) for processor {}",
                self.id()
            )));
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
        let mesh = self.input.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing mesh input for processor {}", self.id()))
        })?;

        let factor = self.factor.ok_or_else(|| {
            ProcessorError::MissingInput(format!(
                "Missing scale factor for processor {}",
                self.id()
            ))
        })?;

        let vertices = mesh
            .vertices
            .iter()
            .map(|v| [v[0] * factor, v[1] * factor, v[2] * factor])
            .collect();

        self.output = Some(Arc::new(Mesh3D::new(vertices, mesh.faces.clone())));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mesh() -> Arc<Mesh3D> {
        Arc::new(Mesh3D::new(
            vec![[1.0, 2.0, 3.0], [-1.0, 0.0, 4.0]],
            vec![[0, 1, 0]],
        ))
    }

    #[test]
    fn test_scale_happy_path() {
        let mut proc = ModelScaleProcessor::new("scale".into());
        proc.set_input(vec![sample_mesh(), Arc::new(2.0f32)])
            .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let mesh = output[0].downcast_ref::<Mesh3D>().unwrap();
        assert_eq!(mesh.vertices[0], [2.0, 4.0, 6.0]);
        assert_eq!(mesh.vertices[1], [-2.0, 0.0, 8.0]);
        // Faces are preserved.
        assert_eq!(mesh.faces[0], [0, 1, 0]);
    }

    #[test]
    fn test_scale_factor_from_string() {
        let mut proc = ModelScaleProcessor::new("scale".into());
        proc.set_input(vec![sample_mesh(), Arc::new("0.5".to_string())])
            .unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let mesh = output[0].downcast_ref::<Mesh3D>().unwrap();
        assert_eq!(mesh.vertices[0], [0.5, 1.0, 1.5]);
    }

    #[test]
    fn test_missing_inputs_fails() {
        let mut proc = ModelScaleProcessor::new("scale".into());
        assert!(matches!(
            proc.set_input(vec![sample_mesh()]).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_wrong_mesh_type_fails() {
        let mut proc = ModelScaleProcessor::new("scale".into());
        let bad: Arc<dyn std::any::Any + Send + Sync> = Arc::new(42u32);
        assert!(matches!(
            proc.set_input(vec![bad, Arc::new(2.0f32)]).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_unparseable_factor_fails() {
        let mut proc = ModelScaleProcessor::new("scale".into());
        assert!(matches!(
            proc.set_input(vec![sample_mesh(), Arc::new("big".to_string())])
                .unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_process_without_input_fails() {
        let mut proc = ModelScaleProcessor::new("scale".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }
}
