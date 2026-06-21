use std::{any::{Any, TypeId}, sync::Arc};

pub trait ProcessorBase: Send + Sync {
    fn input_type_id(&self) -> TypeId;
    fn output_type_id(&self) -> TypeId;
    
    fn process_erased(
        &mut self, 
        input: Option<Arc<dyn Any + Send + Sync>>
    ) -> Result<Arc<dyn Any + Send + Sync>, ProcessorError>;
}

pub trait Processor<I: Send + Sync + 'static, O: Send + Sync + 'static>: ProcessorBase {
    fn process(&mut self, input: Option<Arc<I>>) -> Arc<O>;
}

impl<I, O, P> ProcessorBase for P
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
    P: Processor<I, O>,
{
    fn input_type_id(&self) -> TypeId {
        TypeId::of::<I>()
    }

    fn output_type_id(&self) -> TypeId {
        TypeId::of::<O>()
    }

    fn process_erased(
        &mut self,
        input: Option<Arc<dyn Any + Send + Sync>>
    ) -> Result<Arc<dyn Any + Send + Sync>, ProcessorError>
    {
        let typed_input = input.and_then(|i| i.downcast::<I>().ok());
        let typed_input = match input {
            Some(any_input) => {
                let downcasted = any_input.downcast::<I>()
                    .map_err(|_| ProcessorError::InvalidInput)?;
                Some(downcasted)
            }
            None => None
        };
        let result = self.process(typed_input);
        // .map(|output| output as Arc<dyn Any + Send + Sync>)
        Ok(result)
    }
}

// pub trait Processor {
//     fn input_type(&self) -> TypeId;

//     fn set_input(&mut self, input: Arc<dyn Any + Send + Sync>);
//     fn get_output(&self) -> Option<Arc<dyn Any + Send + Sync>>;
    
//     fn output_type(&self) -> TypeId;

//     fn process(&mut self) -> Result<(), ProcessorError>;
// }

#[derive(Debug)]
pub enum ProcessorError {
    InvalidInput,
    ComputingError(String),
    MissingInput,
}