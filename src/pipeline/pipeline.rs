use crate::processors::base_processor::{ProcessorBase, ProcessorError};
use std::{any::Any, collections::{HashMap, VecDeque}, sync::Arc};

pub enum PipelineErrors {
    UnknownProcessor(String),
    UnknownInputProcessor(String),
    WrongInputType(String),
    ComputingError(String)
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

pub enum InputSource {
    Connection { node_id: String, output_slot: usize },
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

pub struct Pipeline {
    processors: HashMap<String, Box<dyn ProcessorBase>>,
    connections: Vec<NodeConfig>,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            processors: HashMap::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_processor(&mut self, processor: Box<dyn ProcessorBase>, inputs: Vec<InputSource>) {
        let id = processor.id().to_string();
        self.processors.insert(id.clone(), processor);
        self.connections.push(NodeConfig {
            id,
            inputs,
        });
    }

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
                if let InputSource::Connection { node_id: source_id, .. } = source {
                    if !in_degree.contains_key(source_id) {
                        return Err(PipelineErrors::UnknownProcessor(source_id.clone()));
                    }

                    adj_list.get_mut(source_id).unwrap().push(target_id.clone());
                    
                    *in_degree.get_mut(target_id).unwrap() += 1;
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
                    let degree = in_degree.get_mut(neighbor_id).unwrap();
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

    pub fn execute(&mut self) -> Result<(), PipelineErrors> {
        let execution_order = self.plan()?;

        for target_id in execution_order {
            let mut target_processor = self.processors.remove(&target_id)
                .ok_or_else(|| PipelineErrors::UnknownProcessor(target_id.clone()))?;

            let config = self.get_node_config(&target_id)?;
            let mut inputs = Vec::with_capacity(config.inputs.len());

            for source in &config.inputs {
                match source {
                    InputSource::Static(data) => {
                        inputs.push(data.clone());
                    },
                    InputSource::Connection { node_id: source_id, output_slot } => {
                        let source_processor = self.processors.get(source_id)
                            .ok_or_else(|| PipelineErrors::UnknownInputProcessor(source_id.clone()))?;
                        
                        let index = *output_slot;
                        let data = source_processor.get_output_erased()
                            .into_iter()
                            .nth(index)
                            .flatten()
                            .ok_or_else(|| PipelineErrors::ComputingError(format!("Missing output {} for {} processor", index, source_id)))?;
                        
                        inputs.push(data);
                    }
                }
            }

            let result = target_processor.set_input_erased(inputs)
                .map_err(PipelineErrors::from)
                .and_then(|_| target_processor.process().map_err(PipelineErrors::from));

            self.processors.insert(target_id, target_processor);

            result?;
        }

        Ok(())
    }

    fn get_node_config(&self, id: &str) -> Result<&NodeConfig, PipelineErrors> {
        self.connections.iter().find(|c| c.id == id).ok_or(PipelineErrors::UnknownProcessor(id.to_string()))
    }
}