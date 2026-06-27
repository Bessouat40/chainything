use crate::processors::base_processor::{ProcessorBase, ProcessorError};
use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    sync::Arc,
};

/// Represents errors that can occur during pipeline planning or execution.
#[derive(Debug)]
pub enum PipelineErrors {
    UnknownProcessor(String),
    UnknownInputProcessor(String),
    WrongInputType(String),
    ComputingError(String),
}

impl From<ProcessorError> for PipelineErrors {
    fn from(error: ProcessorError) -> Self {
        match error {
            ProcessorError::InvalidInput(msg) => PipelineErrors::WrongInputType(msg),
            ProcessorError::ComputingError(msg) => PipelineErrors::ComputingError(msg),
            ProcessorError::MissingInput(msg) => PipelineErrors::ComputingError(msg),
        }
    }
}

/// Defines the origin of data for a processor's input slot.
pub enum InputSource {
    /// Data sourced from another processor within the same pipeline.
    Connection { node_id: String, output_slot: usize },
    /// Data provided directly as a static value.
    Static(Arc<dyn Any + Send + Sync>),
}

impl InputSource {
    pub fn static_data<T: std::any::Any + Send + Sync>(data: T) -> Self {
        InputSource::Static(Arc::new(data))
    }

    pub fn connection(node_id: impl Into<String>, output_slot: usize) -> Self {
        InputSource::Connection {
            node_id: node_id.into(),
            output_slot,
        }
    }
}

pub struct NodeConfig {
    pub id: String,
    pub inputs: Vec<InputSource>,
}

/// Manages a collection of processors and orchestrates their execution order based on dependencies.
pub struct Pipeline {
    processors: HashMap<String, Box<dyn ProcessorBase>>,
    connections: Vec<NodeConfig>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            processors: HashMap::new(),
            connections: Vec::new(),
        }
    }

    /// Adds a processor to the pipeline and defines its input sources.
    ///
    /// # Arguments
    /// * `processor` - The processor implementation to be added.
    /// * `inputs` - A list of `InputSource` matching the expected input slots of the processor.
    pub fn add_processor(&mut self, processor: Box<dyn ProcessorBase>, inputs: Vec<InputSource>) {
        let id = processor.id().to_string();
        self.processors.insert(id.clone(), processor);
        self.connections.push(NodeConfig { id, inputs });
    }

    /// Performs a topological sort to determine the valid execution sequence of processors.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - A list of processor IDs in correct execution order.
    /// * `Err(PipelineErrors)` - If a cycle is detected or an unknown processor is referenced.
    pub fn plan(&self) -> Result<Vec<String>, PipelineErrors> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        for config in &self.connections {
            in_degree.insert(config.id.clone(), 0);
            adj_list.insert(config.id.clone(), Vec::new());
        }

        for config in &self.connections {
            let target_id = &config.id;

            for source in &config.inputs {
                if let InputSource::Connection {
                    node_id: source_id, ..
                } = source
                {
                    if !in_degree.contains_key(source_id) {
                        return Err(PipelineErrors::UnknownProcessor(source_id.clone()));
                    }

                    let neighbors = adj_list.get_mut(source_id).ok_or_else(|| {
                        PipelineErrors::ComputingError(format!("Corrupted graph: {}", source_id))
                    })?;
                    neighbors.push(target_id.clone());

                    let degree = in_degree.get_mut(target_id).ok_or_else(|| {
                        PipelineErrors::ComputingError(format!("Corrupted graph: {}", target_id))
                    })?;
                    *degree += 1;
                }
            }
        }

        let mut zero_in_degree = VecDeque::new();
        for (id, &degree) in &in_degree {
            if degree == 0 {
                zero_in_degree.push_back(id.clone());
            }
        }

        let mut ordered_plan = Vec::with_capacity(self.connections.len());

        while let Some(current_id) = zero_in_degree.pop_front() {
            ordered_plan.push(current_id.clone());

            if let Some(neighbors) = adj_list.get(&current_id) {
                for neighbor_id in neighbors {
                    let degree = in_degree.get_mut(neighbor_id).ok_or_else(|| {
                        PipelineErrors::ComputingError(format!(
                            "Node missing during topological sort: {}",
                            neighbor_id
                        ))
                    })?;

                    *degree -= 1;

                    if *degree == 0 {
                        zero_in_degree.push_back(neighbor_id.clone());
                    }
                }
            }
        }

        if ordered_plan.len() != self.connections.len() {
            return Err(PipelineErrors::ComputingError(
                "Cycle detected in the pipeline graph! A Directed Acyclic Graph cannot have circular dependencies.".to_string(),
            ));
        }

        Ok(ordered_plan)
    }

    /// Executes all processors in the pipeline based on the topological plan.
    ///
    /// This method resolves input dependencies for each processor and triggers
    /// the `process()` logic sequentially.
    ///
    /// # Errors
    /// * Returns `PipelineErrors` if planning fails, an input is missing,
    ///   or a processor returns an error during execution.
    pub fn execute(&mut self) -> Result<(), PipelineErrors> {
        let execution_order = self.plan()?;

        for target_id in execution_order {
            let mut target_processor = self
                .processors
                .remove(&target_id)
                .ok_or_else(|| PipelineErrors::UnknownProcessor(target_id.clone()))?;

            let config = self.get_node_config(&target_id)?;
            let mut inputs = Vec::with_capacity(config.inputs.len());

            for source in &config.inputs {
                match source {
                    InputSource::Static(data) => {
                        inputs.push(data.clone());
                    }
                    InputSource::Connection {
                        node_id: source_id,
                        output_slot,
                    } => {
                        let source_processor = self.processors.get(source_id).ok_or_else(|| {
                            PipelineErrors::UnknownInputProcessor(source_id.clone())
                        })?;

                        let index = *output_slot;

                        let outputs = source_processor.get_output_erased();
                        let data = outputs.get(index).cloned().ok_or_else(|| {
                            PipelineErrors::ComputingError(format!(
                                "Missing output {} for {} processor",
                                index, source_id
                            ))
                        })?;

                        inputs.push(data);
                    }
                }
            }

            let result = target_processor
                .set_input_erased(inputs)
                .map_err(PipelineErrors::from)
                .and_then(|_| target_processor.process().map_err(PipelineErrors::from));

            self.processors.insert(target_id, target_processor);

            result?;
        }

        Ok(())
    }

    /// Returns every processor's output, keyed by processor id.
    ///
    /// Call after [`execute`](Self::execute). Each entry holds the type-erased
    /// outputs of the corresponding processor, in output-slot order. Useful for
    /// consumers (such as a UI) that want to read results without re-running.
    pub fn collect_outputs(&self) -> HashMap<String, Vec<Arc<dyn Any + Send + Sync>>> {
        self.processors
            .iter()
            .map(|(id, processor)| (id.clone(), processor.get_output_erased()))
            .collect()
    }

    /// Retrieves configuration for a specific node ID.
    fn get_node_config(&self, id: &str) -> Result<&NodeConfig, PipelineErrors> {
        self.connections
            .iter()
            .find(|c| c.id == id)
            .ok_or(PipelineErrors::UnknownProcessor(id.to_string()))
    }
}

struct _MockProcessor {
    id: String,
}

impl ProcessorBase for _MockProcessor {
    fn id(&self) -> &str {
        &self.id
    }
    fn process(&mut self) -> Result<(), ProcessorError> {
        Ok(())
    }
    fn get_output_erased(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        vec![]
    }
    fn set_input_erased(
        &mut self,
        _inputs: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort_valid() {
        let mut pipeline = Pipeline::new();

        // A -> B -> C
        pipeline.add_processor(
            Box::new(_MockProcessor { id: "C".into() }),
            vec![InputSource::connection("B", 0)],
        );
        pipeline.add_processor(Box::new(_MockProcessor { id: "A".into() }), vec![]);
        pipeline.add_processor(
            Box::new(_MockProcessor { id: "B".into() }),
            vec![InputSource::connection("A", 0)],
        );

        let plan = pipeline.plan().unwrap();
        assert_eq!(plan, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut pipeline = Pipeline::new();

        // A -> B -> A
        pipeline.add_processor(
            Box::new(_MockProcessor { id: "A".into() }),
            vec![InputSource::connection("B", 0)],
        );
        pipeline.add_processor(
            Box::new(_MockProcessor { id: "B".into() }),
            vec![InputSource::connection("A", 0)],
        );

        let result = pipeline.plan();
        assert!(matches!(result, Err(PipelineErrors::ComputingError(_))));
    }

    #[test]
    fn test_unknown_processor_reference() {
        let mut pipeline = Pipeline::new();

        pipeline.add_processor(
            Box::new(_MockProcessor { id: "C".into() }),
            vec![InputSource::connection("A", 0)],
        );

        let result = pipeline.plan();
        assert!(matches!(result, Err(PipelineErrors::UnknownProcessor(_))));
    }
}
