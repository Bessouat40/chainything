use std::any::Any;

use crate::processors::base_processor::Processor;

#[derive(Debug)]
pub enum NodeError {
    InvalidInput(String),
    InputNotReady,
    NoInput,
}

pub struct Node {
    processor: Box<dyn Processor<Input = Box<dyn Any>, Output = Box<dyn Any>>>,
    input: Option<Box<Node>>,
    output: Option<Box<dyn Any>>,
    id: u32,
    parameters: Option<std::collections::HashMap<String, Box<dyn std::any::Any>>>,
}

impl Node {
    fn new(processor: Box<dyn Processor<Input = Box<dyn Any>, Output = Box<dyn Any>>>, id: u32) -> Self {
        Node {
            processor,
            input: None,
            output: None,
            id,
            parameters: None,
        }
    }

    fn set_input(&mut self, input_node: Node) {
        self.input = Some(Box::new(input_node));
    }

    fn set_output(&mut self, output: Box<dyn Any>) {
        self.output = Some(output);
    }

    fn execute(&self, output_from_previous_node: &dyn Any) -> Result<(), NodeError> {
        
        if let Some(typed_input) = output_from_previous_node.downcast_ref::<<Self::Processor as Processor>::Input>() {
            
            let result = self.processor.process(typed_input, self.parameters.as_ref());
            
            Ok(())
            
        } else {
            Err(NodeError::InvalidInput(format!("Node {} received wrong input type.", self.id)))
        }
    }
}