use std::{any::{Any, TypeId}, sync::Arc};

pub trait ProcessorBase: Send + Sync + 'static {
    fn input_type_id(&self) -> TypeId;
    fn output_type_id(&self) -> TypeId;
    
    fn set_input_erased(&mut self, input: Arc<dyn Any + Send + Sync>) -> Result<(), ProcessorError>;
    fn get_output_erased(&self) -> Option<Arc<dyn Any + Send + Sync>>;
    fn process(&mut self) -> Result<(), ProcessorError>;
}

pub trait Processor: Send + Sync + 'static {
    type Input: Send + Sync + 'static;
    type Output: Send + Sync + 'static;

    fn set_input(&mut self, input: Arc<Self::Input>);
    fn get_output(&self) -> Option<Arc<Self::Output>>;
    fn process(&mut self) -> Result<(), ProcessorError>;
}

impl<T: Processor> ProcessorBase for T {
    fn input_type_id(&self) -> TypeId {
        TypeId::of::<T::Input>()
    }

    fn output_type_id(&self) -> TypeId {
        TypeId::of::<T::Output>()
    }

    fn set_input_erased(&mut self, input: Arc<dyn Any + Send + Sync>) -> Result<(), ProcessorError> {
        if let Ok(typed_input) = input.downcast::<T::Input>() {
            self.set_input(typed_input);
            Ok(())
        } else {
            Err(ProcessorError::InvalidInput)
        }
    }

    fn get_output_erased(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.get_output().map(|out| out as Arc<dyn Any + Send + Sync>)
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        Processor::process(self)
    }
}

#[derive(Debug)]
pub enum ProcessorError {
    InvalidInput,
    ComputingError(String),
    MissingInput,
}