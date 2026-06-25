# Contributing to Chainything

Thank you for your interest in contributing to Chainything! This guide will walk you through the process of adding a new processor to the `chainything` library.

## Overview

A processor is a unit in the data pipeline that receives typed inputs, performs a computation, and produces typed outputs. All processors implement the `Processor` trait.

## Understanding Input and Output Types

Before creating a processor, understand the type system used in Chainything:

### Existing Shared Types

These types are commonly used across processors and should be reused when compatible:

- **`RawImage`** — RGB or greyscale image representation
  - Fields: `width: u32`, `height: u32`, `pixels: Vec<u8>` (flat row-major buffer)
  - Use this when your processor works with image data
  - Defined in `crates/chainything/src/processors/greyscale_processor.rs`

- **`String`** — File paths or text data
  - Use for file paths, identifiers, or text inputs
  - Example: `ImageReaderProcessor` accepts `Arc<String>` (file path)

### Type-Erasing with `Arc<dyn Any + Send + Sync>`

All inputs and outputs must be wrapped in `Arc<dyn Any + Send + Sync>` to enable dynamic dispatch. Your processor receives and returns type-erased values, then downcasts them to concrete types:

```rust
// Receiving input
let first_input = inputs.remove(0);
if let Ok(typed_image) = first_input.downcast::<RawImage>() {
    self.input = Some(typed_image);
}

// Returning output
self.output
    .clone()
    .into_iter()
    .map(|out| out as Arc<dyn std::any::Any + Send + Sync>)
    .collect()
```

### Creating New Types

If existing types don't fit your processor's needs:

1. **Define the type** in a new module or reuse an existing module (e.g., add to `greyscale_processor.rs` if image-related)
2. **Make it `Clone + Send + Sync + 'static`** so it can be wrapped in `Arc` and type-erased
3. **Document it thoroughly** with doc comments explaining its purpose and fields
4. **Add it to `crates/chainything/src/processors.rs`** if it's meant to be shared (public export)

**Example: Adding a new type**

```rust
/// A histogram representation of image data.
#[derive(Clone)]
pub struct Histogram {
    pub bins: Vec<u32>,
    pub range: (u8, u8),
}
```

## Step 1: Create Your Processor

Create a new file in `crates/chainything/src/processors/` following the naming convention: `your_processor_name.rs`.

### Processor Structure

A processor must:

1. Implement the `Processor` trait
2. Have an `id` field and constructor
3. Implement `set_input()` to accept type-erased inputs and downcast them to concrete types
4. Implement `process()` to perform the computation
5. Implement `get_output()` to return the results as type-erased values
6. Include documentation and tests

## Step 2: Register Your Processor

Add your processor to the registry in `crates/chainything/src/pipeline/registry.rs`:

1. **Import your processor** at the top of the file:

```rust
use crate::processors::your_processor_name::YourProcessor;
```

2. **Register it** in the `with_standard_processors()` method:

```rust
registry.register("YourProcessorName", |id| {
    Ok(Box::new(YourProcessor::new(id)) as Box<dyn ProcessorBase>)
});
```

## Step 3: Export Your Processor

Add your processor to the public exports in `crates/chainything/src/processors.rs`:

```rust
pub mod your_processor_name;
```

Example:

```rust
pub mod blur_processor;
```

## Step 4: Write Tests

**Every processor MUST include tests.** Tests should cover:

- ✅ Happy path: processor works with valid inputs
- ✅ Missing input: processor fails appropriately when inputs are not set
- ✅ Wrong type: processor fails when input types don't match expectations
- ✅ Edge cases: empty images, single pixels, etc.

See the examples in `crates/chainything/src/processors/greyscale_processor.rs` for complete test patterns.

## Step 5: Add Documentation

**All processors must include comprehensive documentation:**

- Document the processor's purpose with a `///` doc comment at the struct level
- Document each method's inputs and outputs
- Document possible errors and their meaning
- Use the format: `**Input:** ... **Output:** ... **Errors:** ...`

See `greyscale_processor.rs` or `image_reader_processor.rs` for examples.

## Step 6: Format Your Code

Before submitting, ensure your code is properly formatted:

```bash
cargo fmt
```

Run this from the repository root to format all Rust files according to the project's style.

## Step 7: Verify Everything Works

Before submitting a PR, run tests to ensure nothing is broken:

```bash
cargo test
```

## Complete Checklist

- [ ] Created processor file in `crates/chainything/src/processors/`
- [ ] Processor implements the `Processor` trait correctly
- [ ] Processor is registered in `crates/chainything/src/pipeline/registry.rs`
- [ ] Processor is exported in `crates/chainything/src/processors.rs`
- [ ] Added comprehensive documentation comments
- [ ] Added tests covering happy path and error cases
- [ ] Ran `cargo fmt` to format code
- [ ] Ran `cargo test` to verify everything works

## Need Help?

- Look at existing processors like `greyscale_processor.rs` or `image_reader_processor.rs` for reference implementations
- Check `base_processor.rs` to understand the `Processor` trait
- Check `registry.rs` for how processors are registered and instantiated

---

# Contributing to the Chainything UI

Once you've created a processor for the library, you should create a corresponding node for the UI layer to allow users to use it in the visual node editor.

## Overview

A node represents a processor (or a source/sink) in the visual DAG editor. Each node maps to a single processor and exposes its inputs and outputs as pins in the UI.

## Step 1: Add Input/Output Types (if needed)

All input and output types are defined in `crates/chainything-ui/src/nodes/base_node.rs` in the `InputOutputType` enum.

### Existing Types

- **`String`** — File paths, text, identifiers
- **`RawImage`** — RGB/greyscale image data

### Adding a New Type

If your processor uses a type not in `InputOutputType`, add it:

```rust
#[derive(Clone)]
pub enum InputOutputType {
    String(String),
    RawImage(Option<RawImage>),
    YourNewType(Option<YourType>),  // Add here
}

impl InputOutputType {
    pub fn to_string(&self) -> &str {
        match self {
            InputOutputType::String(_) => "String",
            InputOutputType::RawImage(_) => "RawImage",
            InputOutputType::YourNewType(_) => "YourNewType",  // Add here
        }
    }
}
```

**Important:** Your type MUST implement `Clone`.

## Step 2: Create Your Node

Create a new file in `crates/chainything-ui/src/nodes/` named `your_processor_name_node.rs`.

### Key Implementation Notes

- **`name()`** must exactly match the processor registry name (or "TextInputNode", "ImageDisplayNode" for non-processor nodes)
- **`is_processor()`** returns `true` if this node wraps a processor, `false` if it's a source/sink
- **`mapping_input()` and `mapping_output()`** define what types are accepted and produced
- **`show_input()` and `show_output()`** are called by egui to render pins; return a `PinInfo` with styling
- **`get_value()`** returns internal state (for source nodes) or `None` (for processor nodes)

## Step 3: Export Your Node

Add your node to `crates/chainything-ui/src/nodes.rs`:

```rust
pub mod your_processor_name_node;
```

## Step 4: Register Your Node

Add your node to `crates/chainything-ui/src/nodes/node_registry.rs`:

1. **Import your node** at the top:

```rust
use crate::nodes::your_processor_name_node::YourNode;
```

2. **Register it** in `create_node_registry()`:

```rust
(
    YourNode::new().name().to_string(),
    Box::new(YourNode::new()) as Box<dyn BaseNode>,
),
```

## Step 5: Verify Input/Output Type Consistency

**Critical:** The node's input/output types **must match** the corresponding processor's expected inputs:

- Node's `mapping_input()` → Types the processor expects
- Node's `mapping_output()` → Types the processor produces

See `payload_parser.rs` for how types are converted between the UI and pipeline execution.

## Step 6: Add Tests and Documentation

- Document the node's purpose and behavior
- Test that pin rendering works correctly
- Ensure type downcasting won't fail at runtime

## Step 7: Format Your Code

```bash
cargo fmt
```

## Complete UI Checklist

- [ ] Created new type in `InputOutputType` (if needed)
- [ ] Created node file in `crates/chainything-ui/src/nodes/`
- [ ] Node implements `BaseNode` trait correctly
- [ ] Input/output types match processor expectations
- [ ] Node exported in `crates/chainything-ui/src/nodes.rs`
- [ ] Node registered in `crates/chainything-ui/src/nodes/node_registry.rs`
- [ ] `name()` matches processor registry name
- [ ] UI pins render correctly (test manually in the app)
- [ ] Ran `cargo fmt`

## Full Contribution Workflow

1. **Create processor** in `crates/chainything/src/processors/` (library contribution steps 1-7)
2. **Register processor** in core registry
3. **Create node** in `crates/chainything-ui/src/nodes/`
4. **Register node** in UI registry
5. **Test end-to-end:** Load the UI, verify the node appears in the palette and works correctly
6. **Submit PR** with both core and UI changes

---

Thank you for contributing! 🎉
