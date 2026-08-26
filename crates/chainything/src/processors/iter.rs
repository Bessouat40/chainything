//! Higher-order processors for iterating a sub-pipeline over a collection.
//!
//! The pipeline boundary is entirely type-erased ([`ErasedItem`]), so a processor
//! that applies a whole sub-pipeline to each element of a list never needs to know
//! the concrete element type. That is what lets [`ForEachProcessor`] reuse the exact
//! same processors a user already has — a sub-pipeline of `Blur -> Resize` is applied
//! per element without rewriting either processor for the collection case.
//!
//! Two pieces cooperate:
//! - [`ItemInputProcessor`] — the sub-pipeline's entry node. It has no wired inputs;
//!   `ForEach` injects the current element into it each iteration.
//! - [`ForEachProcessor`] — rebuilds a fresh sub-pipeline per element and runs them
//!   in parallel (rayon), collecting successes and per-element errors separately.

use std::sync::Arc;

use rayon::prelude::*;

use crate::pipeline_core::builder::{JsonPipelineDef, PipelineBuilder};
use crate::pipeline_core::registry::ProcessorRegistry;
use crate::processors::base_processor::{ErasedItem, ErasedList, Processor, ProcessorError};

/// Entry node of a `ForEach` sub-pipeline: a passthrough for the injected element.
///
/// It declares no inputs in the sub-pipeline definition. On each iteration,
/// [`ForEachProcessor`] injects the current element via
/// [`Pipeline::inject_static`](crate::pipeline_core::pipeline::Pipeline::inject_static),
/// and this node simply forwards it as its output so downstream sub-nodes can consume it.
pub struct ItemInputProcessor {
    id: String,
    value: Option<ErasedItem>,
}

impl ItemInputProcessor {
    pub fn new(id: String) -> Self {
        Self { id, value: None }
    }
}

impl Processor for ItemInputProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, input: Vec<ErasedItem>) -> Result<(), ProcessorError> {
        // The injected element arrives as the first (only) input.
        self.value = input.into_iter().next();
        Ok(())
    }

    fn get_output(&self) -> Vec<ErasedItem> {
        self.value.clone().into_iter().collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        if self.value.is_none() {
            return Err(ProcessorError::MissingInput(format!(
                "ItemInput '{}' received no element to pass through",
                self.id
            )));
        }
        Ok(())
    }
}

/// Terminal node of a `ForEach` sub-pipeline: a passthrough for the element result.
///
/// Symmetric with [`ItemInputProcessor`]. Whatever sub-node feeds this node becomes
/// the per-element result: [`ForEachProcessor`] reads this node's output slot 0 to
/// collect the result for each element. Keeping the terminal explicit (rather than
/// guessing which sub-node is the sink) makes `output_node` unambiguous.
pub struct ItemOutputProcessor {
    id: String,
    value: Option<ErasedItem>,
}

impl ItemOutputProcessor {
    pub fn new(id: String) -> Self {
        Self { id, value: None }
    }
}

impl Processor for ItemOutputProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, input: Vec<ErasedItem>) -> Result<(), ProcessorError> {
        // The wired sub-node result arrives as the first (only) input.
        self.value = input.into_iter().next();
        Ok(())
    }

    fn get_output(&self) -> Vec<ErasedItem> {
        self.value.clone().into_iter().collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        if self.value.is_none() {
            return Err(ProcessorError::MissingInput(format!(
                "ItemOutput '{}' received no result to forward",
                self.id
            )));
        }
        Ok(())
    }
}

/// Records why a single element failed, keeping its position in the input collection.
#[derive(Debug, Clone)]
pub struct ItemError {
    pub index: usize,
    pub message: String,
}

/// Applies a sub-pipeline to every element of an input collection.
///
/// # Inputs
/// - `inputs[0]` = `Arc<ErasedList>` — the collection to iterate (produced by any
///   generator processor whose output slot holds a `Vec<ErasedItem>`).
///
/// # Outputs
/// - slot 0 = `Arc<ErasedList>` — the sub-pipeline results for the elements that
///   succeeded, in input order (failed elements are omitted).
/// - slot 1 = `Arc<Vec<ItemError>>` — one entry per failed element (partial-failure
///   report). Empty when every element succeeded.
///
/// # Execution
/// Because [`Pipeline`](crate::pipeline_core::pipeline::Pipeline) instances are not
/// clonable, a fresh sub-pipeline is built from `sub_def` for each element. That has a
/// happy side effect: elements share no mutable state, so they run in parallel with
/// rayon for free.
pub struct ForEachProcessor {
    id: String,
    /// Shared registry used to rebuild the sub-pipeline per element.
    registry: Arc<ProcessorRegistry>,
    /// Definition of the sub-pipeline drawn in the UI (or authored in JSON).
    sub_def: JsonPipelineDef,
    /// Id of the [`ItemInputProcessor`] node that receives each element.
    input_node: String,
    /// Id of the sub-node whose output is collected as this element's result.
    output_node: String,
    /// Output slot to read on `output_node`.
    output_slot: usize,
    items: Option<Arc<ErasedList>>,
    output: Option<Arc<ErasedList>>,
    errors: Vec<ItemError>,
}

impl ForEachProcessor {
    pub fn new(
        id: String,
        registry: Arc<ProcessorRegistry>,
        sub_def: JsonPipelineDef,
        input_node: String,
        output_node: String,
        output_slot: usize,
    ) -> Self {
        Self {
            id,
            registry,
            sub_def,
            input_node,
            output_node,
            output_slot,
            items: None,
            output: None,
            errors: Vec::new(),
        }
    }

    /// Builds one fresh sub-pipeline, feeds it a single element, runs it, and returns
    /// the requested output slot. Errors are stringified so they can cross the rayon
    /// boundary and land in the partial-failure report.
    fn run_one(&self, item: &ErasedItem) -> Result<ErasedItem, String> {
        let mut sub = PipelineBuilder::build_pipeline(&self.sub_def, &self.registry)
            .map_err(|e| format!("sub-pipeline build failed: {:?}", e))?;

        sub.inject_static(&self.input_node, item.clone())
            .map_err(|e| format!("could not inject element into '{}': {:?}", self.input_node, e))?;

        sub.execute()
            .map_err(|e| format!("sub-pipeline execution failed: {:?}", e))?;

        let outputs = sub.collect_outputs();
        let node_out = outputs.get(&self.output_node).ok_or_else(|| {
            format!("output node '{}' not found in sub-pipeline", self.output_node)
        })?;
        node_out.get(self.output_slot).cloned().ok_or_else(|| {
            format!(
                "output slot {} missing on node '{}'",
                self.output_slot, self.output_node
            )
        })
    }
}

impl Processor for ForEachProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, inputs: Vec<ErasedItem>) -> Result<(), ProcessorError> {
        let first = inputs.into_iter().next().ok_or_else(|| {
            ProcessorError::MissingInput(format!(
                "ForEach '{}' expects one input: the collection to iterate",
                self.id
            ))
        })?;

        let list = first.downcast::<ErasedList>().map_err(|_| {
            ProcessorError::InvalidInput(format!(
                "ForEach '{}' expects a list (Arc<Vec<ErasedItem>>) as input",
                self.id
            ))
        })?;

        self.items = Some(list);
        Ok(())
    }

    fn get_output(&self) -> Vec<ErasedItem> {
        let successes: ErasedItem = match &self.output {
            Some(list) => list.clone() as ErasedItem,
            None => Arc::new(ErasedList::new()) as ErasedItem,
        };
        let errors: ErasedItem = Arc::new(self.errors.clone()) as ErasedItem;
        vec![successes, errors]
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let items = self.items.clone().ok_or_else(|| {
            ProcessorError::MissingInput(format!("ForEach '{}' has no input collection set", self.id))
        })?;

        // Parallel map: each element gets its own sub-pipeline instance, so there is
        // no shared mutable state to guard. Order is preserved by rayon's collect.
        let outcomes: Vec<Result<ErasedItem, String>> =
            items.par_iter().map(|item| self.run_one(item)).collect();

        let mut successes: ErasedList = Vec::new();
        let mut errors: Vec<ItemError> = Vec::new();
        for (index, outcome) in outcomes.into_iter().enumerate() {
            match outcome {
                Ok(value) => successes.push(value),
                Err(message) => errors.push(ItemError { index, message }),
            }
        }

        self.output = Some(Arc::new(successes));
        self.errors = errors;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::base_processor::ProcessorBase;

    /// Test processor: doubles an i32, but errors on negatives.
    struct DoubleProcessor {
        id: String,
        input: Option<i32>,
        output: Option<i32>,
    }

    impl DoubleProcessor {
        fn new(id: String) -> Self {
            Self { id, input: None, output: None }
        }
    }

    impl Processor for DoubleProcessor {
        fn id(&self) -> &str {
            &self.id
        }

        fn set_input(&mut self, input: Vec<ErasedItem>) -> Result<(), ProcessorError> {
            let first = input
                .into_iter()
                .next()
                .ok_or_else(|| ProcessorError::MissingInput("Double needs one input".into()))?;
            self.input = Some(
                *first
                    .downcast_ref::<i32>()
                    .ok_or_else(|| ProcessorError::InvalidInput("expected i32".into()))?,
            );
            Ok(())
        }

        fn get_output(&self) -> Vec<ErasedItem> {
            self.output
                .map(|v| vec![Arc::new(v) as ErasedItem])
                .unwrap_or_default()
        }

        fn process(&mut self) -> Result<(), ProcessorError> {
            let v = self
                .input
                .ok_or_else(|| ProcessorError::MissingInput("Double has no input".into()))?;
            if v < 0 {
                return Err(ProcessorError::ComputingError("negative not allowed".into()));
            }
            self.output = Some(v * 2);
            Ok(())
        }
    }

    fn test_registry() -> Arc<ProcessorRegistry> {
        let mut reg = ProcessorRegistry::new();
        reg.register("ItemInput", |id| {
            Ok(Box::new(ItemInputProcessor::new(id)) as Box<dyn ProcessorBase>)
        });
        reg.register("Double", |id| {
            Ok(Box::new(DoubleProcessor::new(id)) as Box<dyn ProcessorBase>)
        });
        Arc::new(reg)
    }

    fn sub_def() -> JsonPipelineDef {
        serde_json::from_str(
            r#"{"nodes":[
                {"id":"item","type":"ItemInput","inputs":[]},
                {"id":"double","type":"Double","inputs":[{"source_node":"item","source_slot":0}]}
            ]}"#,
        )
        .unwrap()
    }

    fn list(values: &[i32]) -> ErasedItem {
        let items: ErasedList = values.iter().map(|&v| Arc::new(v) as ErasedItem).collect();
        Arc::new(items) as ErasedItem
    }

    #[test]
    fn test_foreach_maps_every_element() {
        let mut fe = ForEachProcessor::new(
            "fe".into(),
            test_registry(),
            sub_def(),
            "item".into(),
            "double".into(),
            0,
        );
        fe.set_input(vec![list(&[1, 2, 3])]).unwrap();
        Processor::process(&mut fe).unwrap();

        let out = fe.get_output();
        let successes = out[0].downcast_ref::<ErasedList>().unwrap();
        let got: Vec<i32> = successes
            .iter()
            .map(|v| *v.downcast_ref::<i32>().unwrap())
            .collect();
        assert_eq!(got, vec![2, 4, 6]);

        let errors = out[1].downcast_ref::<Vec<ItemError>>().unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_foreach_collects_per_element_errors() {
        let mut fe = ForEachProcessor::new(
            "fe".into(),
            test_registry(),
            sub_def(),
            "item".into(),
            "double".into(),
            0,
        );
        // Element at index 1 (-1) fails; the rest still succeed.
        fe.set_input(vec![list(&[1, -1, 3])]).unwrap();
        Processor::process(&mut fe).unwrap();

        let out = fe.get_output();
        let successes = out[0].downcast_ref::<ErasedList>().unwrap();
        let got: Vec<i32> = successes
            .iter()
            .map(|v| *v.downcast_ref::<i32>().unwrap())
            .collect();
        assert_eq!(got, vec![2, 6]);

        let errors = out[1].downcast_ref::<Vec<ItemError>>().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].index, 1);
    }

    #[test]
    fn test_foreach_rejects_non_list_input() {
        let mut fe = ForEachProcessor::new(
            "fe".into(),
            test_registry(),
            sub_def(),
            "item".into(),
            "double".into(),
            0,
        );
        let bad = Arc::new(42i32) as ErasedItem;
        assert!(matches!(
            fe.set_input(vec![bad]),
            Err(ProcessorError::InvalidInput(_))
        ));
    }
}
