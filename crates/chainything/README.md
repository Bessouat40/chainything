# Chainything

A Rust-based DAG (Directed Acyclic Graph) pipeline execution engine with strong typing and automatic topological sorting.

This is the **core library** of Chainything. For the visual node editor, see [chainything-ui](https://crates.io/crates/chainything-ui).

## Features

- **Automatic Topological Sorting:** Uses Kahn's algorithm to determine the execution order of processors and detect circular dependencies safely.
- **Strong Typing & Flexibility:** Processors strictly define their input and output types, while the pipeline manages data transfer dynamically via type erasure (Any).
- **Multiple Sources:** Nodes can receive static data (provided at startup) or dynamic data (coming from the output of another node).
- **Extensible:** Simply implement the Processor trait to create your own custom logic blocks.

## Quick Start

```rust
use chainything::prelude::*;

fn main() {
    // 1. Initialize the pipeline
    let mut pipeline = Pipeline::new();

    // 2. Add the reader processor (static data input)
    let reader = ImageReaderProcessor::new("reader");
    pipeline.add_processor(
        Box::new(reader),
        vec![InputSource::static_data("./cat.jpg")]
    );

    // 3. Add the grayscale processor (connected to output 0 of "reader")
    let greyscale = GreyScaleProcessor::new("greyscale");
    pipeline.add_processor(
        Box::new(greyscale),
        vec![InputSource::connection("reader", 0)]
    );

    // 4. Add the saver processor (connected to output 0 of "greyscale")
    let saver = ImageSaveProcessor::new("saver", "./output.png");
    pipeline.add_processor(
        Box::new(saver),
        vec![InputSource::connection("greyscale", 0)]
    );

    // 5. Execute the DAG
    match pipeline.execute() {
        Ok(_) => println!("Pipeline executed successfully!"),
        Err(e) => eprintln!("Execution error: {:?}", e),
    }
}
```

## Architecture

The pipeline is built around three core concepts:

- **Processor:** A trait you implement to define a logical unit of work (e.g., reading a file, applying a filter).
- **InputSource:** Defines where the data comes from (Static for hardcoded values, Connection to link to another node's output slot).
- **Pipeline:** The orchestrator that registers processors, analyzes their connections, and executes them in the correct order.

## Creating Your Own Processor

To create a new processor, implement the `Processor` trait:

```rust
impl Processor for MyProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, inputs: Vec<Arc<dyn Any + Send + Sync>>) -> Result<(), ProcessorError> {
        // Downcast and store inputs
        Ok(())
    }

    fn get_output(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        vec![Arc::new("result".to_string())]
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        // Your logic here
        Ok(())
    }
}
```

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for detailed guidance.

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines on adding new processors.
