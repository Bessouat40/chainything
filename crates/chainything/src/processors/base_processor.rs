use std::{any::Any, sync::Arc};

/// Type-erased counterpart to [`Processor`], enabling dynamic dispatch in heterogeneous pipelines.
///
/// Automatically implemented for any type that implements [`Processor`] via a blanket impl.
/// Prefer implementing [`Processor`] directly — this trait is for runtime polymorphism only.
pub trait ProcessorBase: Send + Sync + 'static {
    /// Unique identifier for this processor.
    fn id(&self) -> &str;

    /// Sets inputs as type-erased `Arc` values.
    ///
    /// - **Input:** `Vec<Arc<dyn Any + Send + Sync>>` — values to downcast to concrete types.
    /// - **Errors:** [`ProcessorError::InvalidInput`] if a downcast fails,
    ///   [`ProcessorError::MissingInput`] if a required value is absent.
    fn set_input_erased(
        &mut self,
        input: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), ProcessorError>;

    /// Returns outputs as type-erased `Arc` values. Call after [`process`](Self::process).
    fn get_output_erased(&self) -> Vec<Arc<dyn Any + Send + Sync>>;

    /// Runs the core computation. Call after [`set_input_erased`](Self::set_input_erased).
    ///
    /// - **Errors:** [`ProcessorError::MissingInput`] if inputs were not set,
    ///   [`ProcessorError::ComputingError`] if the computation fails.
    fn process(&mut self) -> Result<(), ProcessorError>;
}

/// A typed node in a data pipeline.
///
/// Implement this to define a unit that receives typed inputs, computes a result,
/// and exposes typed outputs. The blanket `impl<T: Processor> ProcessorBase for T`
/// bridges this to [`ProcessorBase`] for use in dynamic pipelines.
///
/// # Lifecycle
/// 1. Call [`set_input`](Self::set_input) with the required inputs.
/// 2. Call [`process`](Self::process) to run the computation.
/// 3. Call [`get_output`](Self::get_output) to retrieve the results.
pub trait Processor: Send + Sync + 'static {
    /// Unique identifier for this processor.
    fn id(&self) -> &str;

    /// Sets the typed inputs for this processor.
    ///
    /// - **Input:** `Vec<Arc<dyn Any + Send + Sync>>` — each value should be downcast
    ///   to its expected concrete type inside this method.
    /// - **Errors:** [`ProcessorError::InvalidInput`] if a value cannot be cast,
    ///   [`ProcessorError::MissingInput`] if a required value is absent.
    fn set_input(&mut self, input: Vec<Arc<dyn Any + Send + Sync>>) -> Result<(), ProcessorError>;

    /// Returns the computed output values. Call after [`process`](Self::process) succeeds.
    fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>>;

    /// Runs the processor's computation.
    ///
    /// - **Errors:** [`ProcessorError::MissingInput`] if inputs were not set,
    ///   [`ProcessorError::ComputingError`] if the computation fails.
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Adder {
        a: Option<i32>,
        b: Option<i32>,
        result: Option<i32>,
    }

    impl Adder {
        fn new() -> Self {
            Self {
                a: None,
                b: None,
                result: None,
            }
        }
    }

    impl Processor for Adder {
        fn id(&self) -> &str {
            "adder"
        }

        fn set_input(
            &mut self,
            input: Vec<Arc<dyn Any + Send + Sync>>,
        ) -> Result<(), ProcessorError> {
            self.a =
                Some(*input[0].downcast_ref::<i32>().ok_or_else(|| {
                    ProcessorError::InvalidInput("input[0]: expected i32".into())
                })?);
            self.b =
                Some(*input[1].downcast_ref::<i32>().ok_or_else(|| {
                    ProcessorError::InvalidInput("input[1]: expected i32".into())
                })?);
            Ok(())
        }

        fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
            vec![Arc::new(self.result.unwrap_or(0))]
        }

        fn process(&mut self) -> Result<(), ProcessorError> {
            let a = self
                .a
                .ok_or_else(|| ProcessorError::MissingInput("a".into()))?;
            let b = self
                .b
                .ok_or_else(|| ProcessorError::MissingInput("b".into()))?;
            self.result = Some(a + b);
            Ok(())
        }
    }

    #[test]
    fn test_process_happy_path() {
        let mut adder = Adder::new();
        adder
            .set_input(vec![Arc::new(3i32), Arc::new(4i32)])
            .unwrap();
        Processor::process(&mut adder).unwrap();

        let output = adder.get_output();
        assert_eq!(*output[0].downcast_ref::<i32>().unwrap(), 7);
    }

    #[test]
    fn test_process_without_input_returns_missing_input_error() {
        let mut adder = Adder::new();
        assert!(matches!(
            Processor::process(&mut adder).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_processor_base_via_blanket_impl() {
        let mut adder: Box<dyn ProcessorBase> = Box::new(Adder::new());
        adder
            .set_input_erased(vec![Arc::new(10i32), Arc::new(5i32)])
            .unwrap();
        adder.process().unwrap();

        let output = adder.get_output_erased();
        assert_eq!(*output[0].downcast_ref::<i32>().unwrap(), 15);
    }
}
