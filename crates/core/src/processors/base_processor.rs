use std::{any::Any, sync::Arc};

pub trait ProcessorBase: Send + Sync + 'static {
    fn id(&self) -> &str;

    fn set_input_erased(
        &mut self,
        input: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), ProcessorError>;
    fn get_output_erased(&self) -> Vec<Arc<dyn Any + Send + Sync>>;
    fn process(&mut self) -> Result<(), ProcessorError>;
}

pub trait Processor: Send + Sync + 'static {
    fn id(&self) -> &str;

    fn set_input(&mut self, input: Vec<Arc<dyn Any + Send + Sync>>) -> Result<(), ProcessorError>;
    fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>>;
    fn process(&mut self) -> Result<(), ProcessorError>;
}

impl<T: Processor> ProcessorBase for T {
    fn id(&self) -> &str {
        Processor::id(self)
    }

    fn set_input_erased(
        &mut self,
        input: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        self.set_input(input)
    }

    fn get_output_erased(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.get_output()
            .into_iter()
            .map(|out| out as Arc<dyn Any + Send + Sync>)
            .collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        Processor::process(self)
    }
}

#[derive(Debug)]
pub enum ProcessorError {
    InvalidInput(String),
    ComputingError(String),
    MissingInput(String),
}
