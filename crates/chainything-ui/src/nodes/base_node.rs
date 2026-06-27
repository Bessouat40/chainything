use dyn_clone::DynClone;
use std::collections::HashMap;

use chainything::processors::greyscale_processor::RawImage;
use egui::{Color32, Ui};
use egui_snarl::{InPin, NodeId, OutPin, Snarl, ui::PinInfo};

pub const STRING_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);

#[derive(Clone)]
pub enum InputOutputType {
    String(String),
    RawImage(Option<RawImage>),
}

impl InputOutputType {
    pub fn to_string(&self) -> &str {
        match self {
            InputOutputType::String(_) => "String",
            InputOutputType::RawImage(_) => "RawImage",
        }
    }
}

pub trait BaseNode: DynClone {
    fn name(&self) -> &str;
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
}

dyn_clone::clone_trait_object!(BaseNode);
