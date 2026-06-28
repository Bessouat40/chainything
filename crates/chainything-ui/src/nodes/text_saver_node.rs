use std::cell::RefCell;
use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, InputOutputType, NodeCategory, NodeInformations, STRING_COLOR,
};

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

/// Visual node wrapping the `TextSave` processor.
///
/// Writes the text received on its single input pin to the file path configured
/// in the node body. Acts as a sink for text-producing nodes such as
/// [`OllamaNode`](crate::nodes::ollama_node::OllamaNode).
#[derive(Clone)]
pub struct TextSaveNode {
    path_input: RefCell<String>,
}

impl TextSaveNode {
    pub fn new() -> Self {
        Self {
            path_input: RefCell::new("".to_string()),
        }
    }
}

impl BaseNode for TextSaveNode {
    fn name(&self) -> &str {
        "TextSave"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new("Writes the incoming text to the configured file path on disk.")
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Text
    }

    fn is_processor(&self) -> bool {
        true
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        None
    }

    fn outputs_count(&self) -> usize {
        0
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn inputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(
            0,
            InputOutputType::String("".to_string()),
        )]))
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Text");
            ui.add_space(5.0);
        });

        PinInfo::circle()
            .with_fill(STRING_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn show_output(&mut self, _pin: &OutPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
    }

    fn has_body(&self) -> bool {
        true
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(40, 70, 40))
    }

    fn show_body(
        &self,
        _node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _snarl: &Snarl<Box<dyn BaseNode>>,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(200.0);

            ui.horizontal(|ui| {
                ui.label("Save To");
                let mut text = self.path_input.borrow().clone();

                if ui.text_edit_singleline(&mut text).changed() {
                    *self.path_input.borrow_mut() = text;
                }
            });
        });
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.path_input.borrow().clone()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        if index == 0 {
            *self.path_input.borrow_mut() = value.to_string();
        }
    }
}
