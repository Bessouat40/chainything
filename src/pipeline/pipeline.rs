use crate::{processors::base_processor::Processor};
use std::collections::HashMap;

enum PipelineErrors {
    UnknownProcessor,
    UnknownInputProcessor,
    WrongInputType,
    ComputingError
}

pub struct Pipeline {
    pub processors: HashMap<String, Box<dyn Processor>>,
    links: HashMap<String, Vec<String>>
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            processors: HashMap::new(),
            links: HashMap::new() }
    }

    pub fn add_processor(&mut self, id: &str, processor: Box<dyn Processor>, input_processor_id: Option<&str> ) {
        self.processors.insert(
            String::from(id),
            processor
        );
        let inputs: Vec<String>;
        if let Some(input_processor_id) = input_processor_id {
            inputs = vec![String::from(input_processor_id)];
        } else {
            inputs = vec![];
        }
        self.links.insert(
            String::from(id),
            inputs
        );
    }

    fn execute_processor(&mut self, processor_id: &String) -> Result<(), PipelineErrors> {
        let input = {
            let input_id = self.links.get(processor_id)
                .and_then(|ids| ids.first())
                .cloned();

            if let Some(input_id) = input_id {
                let input_processor = self.processors.get(&input_id)
                    .ok_or(PipelineErrors::UnknownInputProcessor)?;
                
                match input_processor.get_output() {
                    Some(output) => Some(output),
                    None => {
                        drop(input_processor);
                        self.execute_processor(&input_id)?;
                        self.processors.get(&input_id)
                            .and_then(|p| p.get_output())
                    }
                }
            } else {
                None
            }
        };

        let processor = self.processors.get_mut(processor_id)
            .ok_or(PipelineErrors::UnknownProcessor)?;

        if let Some(input) = input {
            processor.set_input(input);
            processor.process().map_err(|_| PipelineErrors::ComputingError)?;
        }

        Ok(())
    }

    pub fn execute(&mut self) -> Result<(), String> {
        let ids: Vec<String> = self.processors.keys().cloned().collect();
        for id in ids {
            self.execute_processor(&id);
        }
        Ok(())
    }
}