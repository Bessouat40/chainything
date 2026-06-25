# Chainything UI

A visual node editor for the [Chainything](https://crates.io/crates/chainything) DAG pipeline engine, built with egui.

This package provides an interactive graphical interface to create and execute data processing pipelines without writing code.

## Features

- **Visual Node Editor:** Drag-and-drop interface to build DAG pipelines
- **Instant Feedback:** See pipeline topology visualized in real-time
- **Execute Pipelines:** Run your pipeline with a single click
- **Extensible:** Built on the Chainything core library for easy processor integration

## Getting Started

### From Repository

```bash
git clone https://github.com/Bessouat40/chainything
cd chainything
cargo run --package chainything-ui
```

### As a Library

```toml
[dependencies]
chainything-ui = "0.1.0"
```

## Usage

1. **Add Nodes:** Right-click on the canvas to add processor nodes
2. **Connect Nodes:** Drag from output pins to input pins to create data flow
3. **Configure:** Click on nodes to set parameters (e.g., file paths)
4. **Execute:** Click "Run" to execute the entire pipeline in topological order

## Building UI Nodes

To create a UI node for your processor:

1. Create a node file in `src/nodes/your_processor_node.rs`
2. Implement the `BaseNode` trait
3. Register it in `src/nodes/node_registry.rs`

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for detailed instructions.

## Architecture

The UI is built with:

- **egui:** Immediate-mode GUI framework
- **egui-snarl:** Node graph visualization
- **Chainything Core:** The pipeline execution engine

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines on adding new UI nodes.
