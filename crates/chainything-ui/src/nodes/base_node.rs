use dyn_clone::DynClone;
use std::collections::HashMap;

use chainything::processors::images::greyscale_processor::RawImage;
use chainything::processors::model3d::mesh::Mesh3D;
use egui::{Color32, Ui};
use egui_snarl::{InPin, NodeId, OutPin, Snarl, ui::PinInfo};

pub const STRING_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);
pub const LLM_COLOR: Color32 = Color32::from_rgb(0xd0, 0x80, 0x20);
pub const MESH_COLOR: Color32 = Color32::from_rgb(0x40, 0x90, 0xd0);
/// Pin colour for a materialized collection (`Vec` of elements).
pub const LIST_COLOR: Color32 = Color32::from_rgb(0xc0, 0x60, 0xe0);
/// Pin colour for a single loop element (wildcard, connects to any type).
pub const ITEM_COLOR: Color32 = Color32::from_rgb(0xe0, 0xc0, 0x40);

#[derive(Clone)]
pub enum InputOutputType {
    String(String),
    RawImage(Option<RawImage>),
    /// A loaded language model handle, produced by a provider loader node and
    /// consumed by generation nodes. Carries no UI-side data — it only exists
    /// at pipeline-execution time.
    Llm,
    /// A 3D triangle mesh, produced by a model reader and consumed by transform
    /// or save nodes.
    Mesh3D(Option<Mesh3D>),
    /// A materialized collection, produced by a generator node and consumed by a
    /// [`ForEach`](super::iter::foreach_node::ForEachNode) node.
    List,
    /// A single element flowing inside a `ForEach` loop body. It is a wildcard:
    /// the concrete element type is only known at run time, so it may connect to a
    /// pin of any type (see [`InputOutputType::connects_to`]).
    Item,
}

impl InputOutputType {
    pub fn to_string(&self) -> &str {
        match self {
            InputOutputType::String(_) => "String",
            InputOutputType::RawImage(_) => "RawImage",
            InputOutputType::Llm => "LLM",
            InputOutputType::Mesh3D(_) => "Mesh3D",
            InputOutputType::List => "List",
            InputOutputType::Item => "Item",
        }
    }

    /// Whether an output of type `self` may feed an input of type `other`.
    ///
    /// Types must match by discriminant, except that [`Item`](InputOutputType::Item)
    /// is a wildcard on either side: the element leaving an `ItemInput` (or entering
    /// an `ItemOutput`) has a run-time-only type, so it connects to any pin.
    pub fn connects_to(&self, other: &InputOutputType) -> bool {
        use std::mem::discriminant;
        matches!(self, InputOutputType::Item)
            || matches!(other, InputOutputType::Item)
            || discriminant(self) == discriminant(other)
    }
}

/// Human-readable documentation for a node, shown in the info modal opened from
/// the node header. Input and output types are derived separately from the
/// node's pin mappings, so this only carries the prose description.
#[derive(Clone)]
pub struct NodeInformations {
    pub description: String,
}

impl NodeInformations {
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
        }
    }
}

/// Broad family a node belongs to, used to group nodes by data domain in the
/// library panel.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeCategory {
    Text,
    Image,
    Llm,
    Model3D,
    /// Control-flow nodes: `ForEach` and its sub-pipeline markers.
    Flow,
}

impl NodeCategory {
    /// Header label shown for the category in the library panel.
    pub fn label(&self) -> &'static str {
        match self {
            NodeCategory::Text => "TEXT",
            NodeCategory::Image => "IMAGE",
            NodeCategory::Llm => "LLM",
            NodeCategory::Model3D => "3D",
            NodeCategory::Flow => "FLOW",
        }
    }

    /// Categories in the order they should appear in the library panel.
    pub const ALL: [NodeCategory; 5] = [
        NodeCategory::Text,
        NodeCategory::Image,
        NodeCategory::Llm,
        NodeCategory::Model3D,
        NodeCategory::Flow,
    ];
}

/// Runtime data pushed into a display node after a pipeline run, so it can be
/// visualized directly in the graph without saving to disk first.
#[derive(Clone)]
pub enum DisplayData {
    Text(String),
    Image(RawImage),
}

pub trait BaseNode: DynClone {
    fn name(&self) -> &str;
    /// Documentation shown in the node's info modal (description only; input and
    /// output types are derived from the pin mappings).
    fn informations(&self) -> NodeInformations;
    /// Data domain this node belongs to, used to group it in the library panel.
    fn category(&self) -> NodeCategory;
    fn inputs_count(&self) -> usize;
    fn is_processor(&self) -> bool;
    fn outputs_count(&self) -> usize;
    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>>;
    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>>;
    fn show_input(&mut self, pin: &InPin, ui: &mut Ui) -> PinInfo;
    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui) -> PinInfo;
    fn has_body(&self) -> bool;
    fn get_value(&self) -> Option<&Vec<InputOutputType>>;
    fn header_frame(&self, frame: egui::Frame) -> egui::Frame;
    fn show_body(
        &self,
        _node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        _ui: &mut Ui,
        _snarl: &Snarl<Box<dyn BaseNode>>,
    ) {
    }
    fn get_parameter(&self, _index: usize) -> Option<String> {
        None
    }

    /// Restores a parameter previously read via [`get_parameter`], used when
    /// importing a graph from JSON. `index` matches the one used by
    /// `get_parameter`. The default is a no-op so nodes without editable state
    /// (e.g. display nodes) need no implementation.
    fn set_parameter(&mut self, _index: usize, _value: &str) {}

    /// Receives runtime data produced upstream, after a pipeline run.
    ///
    /// Display nodes override this to store the value (using interior
    /// mutability) and render it. The default is a no-op.
    fn set_display(&self, _data: DisplayData) {}

    /// Drops any runtime data previously pushed via [`set_display`].
    ///
    /// Called on every display node at the start of a run so stale results
    /// (and their GPU textures) are freed and the graph shows fresh output.
    /// The default is a no-op.
    fn clear_display(&self) {}

    /// Whether this node hosts an editable sub-pipeline (a nested graph).
    ///
    /// Only `ForEach` returns `true`. It makes the node menu offer "Edit loop
    /// body" and lets the editor drill into the nested graph. The default is
    /// `false`, so no existing node is affected.
    fn has_sub_editor(&self) -> bool {
        false
    }

    /// The nested sub-pipeline graph, if this node hosts one.
    ///
    /// Used by the payload exporter and the on-disk serializer to recurse into
    /// the loop body. The default is `None`.
    fn sub_snarl(&self) -> Option<&Snarl<Box<dyn BaseNode>>> {
        None
    }

    /// Mutable access to the nested sub-pipeline graph, if any.
    ///
    /// Used by the editor to drill into the loop body and by the importer to
    /// rebuild it. The default is `None`.
    fn sub_snarl_mut(&mut self) -> Option<&mut Snarl<Box<dyn BaseNode>>> {
        None
    }
}

dyn_clone::clone_trait_object!(BaseNode);
