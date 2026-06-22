use std::{any::{Any, TypeId}, sync::Arc};
pub trait Processor {
    type Input: Send + Sync + 'static;
    type Output: Send + Sync + 'static;
    const INPUT: Option<Arc<Self::Input>>;
    const OUTPUT: Option<Arc<Self::Output>>;

    fn set_input(&mut self, input: Arc<dyn Any + Send + Sync>);
    fn get_output(&self) -> Option<Arc<dyn Any + Send + Sync>>;

    fn input_type_id(&self) -> TypeId {
        TypeId::of::<Self::Input>()
    }

    fn output_type_id(&self) -> TypeId {
        TypeId::of::<Self::Output>()
    }

    fn process(&mut self) -> Result<(), ProcessorError>;
}

#[derive(Debug)]
pub enum ProcessorError {
    InvalidInput,
    ComputingError(String),
    MissingInput,
}