use dyn_clone::DynClone;
use std::collections::HashMap;

use chainything::processors::greyscale_processor::RawImage;
use egui::{Color32, Ui};
use egui_snarl::{InPin, NodeId, OutPin, Snarl, ui::PinInfo};

pub const STRING_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);
pub const LLM_COLOR: Color32 = Color32::from_rgb(0xd0, 0x80, 0x20);

#[derive(Clone)]
pub enum InputOutputType {
    String(String),
    RawImage(Option<RawImage>),
    /// A loaded language model handle, produced by a provider loader node and
    /// consumed by generation nodes. Carries no UI-side data — it only exists
    /// at pipeline-execution time.
    Llm,
}

impl InputOutputType {
    pub fn to_string(&self) -> &str {
        match self {
            InputOutputType::String(_) => "String",
            InputOutputType::RawImage(_) => "RawImage",
            InputOutputType::Llm => "LLM",
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
}

impl NodeCategory {
    /// Header label shown for the category in the library panel.
    pub fn label(&self) -> &'static str {
        match self {
            NodeCategory::Text => "TEXT",
            NodeCategory::Image => "IMAGE",
            NodeCategory::Llm => "LLM",
        }
    }

    /// Categories in the order they should appear in the library panel.
    pub const ALL: [NodeCategory; 3] =
        [NodeCategory::Text, NodeCategory::Image, NodeCategory::Llm];
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
}

dyn_clone::clone_trait_object!(BaseNode);
