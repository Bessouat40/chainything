use std::{any::{Any, TypeId}, sync::Arc};

pub trait Processor {
    fn input_type(&self) -> TypeId;

    fn set_input(&mut self, input: Arc<dyn Any + Send + Sync>);
    fn get_output(&self) -> Option<Arc<dyn Any + Send + Sync>>;
    
    fn output_type(&self) -> TypeId;

    fn process(&mut self) -> Result<(), ProcessorError>;
}

#[derive(Debug)]
pub enum ProcessorError {
    InvalidInput(String),
    ComputingError(String),
    MissingInput,
}