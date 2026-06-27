use std::cell::RefCell;
use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, LLM_COLOR, NodeCategory};

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

/// Visual node wrapping the `OllamaLoader` processor.
///
/// Produces a reusable `LLM` handle on its single output pin. The model name is
/// configured in the node body and sent to the processor as a parameter.
/// Connect the output to any node that consumes an `LLM` (e.g. `LLMGenerate`).
#[derive(Clone)]
pub struct OllamaLoaderNode {
    model: RefCell<String>,
}

impl OllamaLoaderNode {
    pub fn new() -> Self {
        Self {
            model: RefCell::new("llama3.2".to_string()),
        }
    }
}

impl BaseNode for OllamaLoaderNode {
    fn name(&self) -> &str {
        "OllamaLoader"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Llm
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        None
    }

    fn is_processor(&self) -> bool {
        true
    }

    fn inputs_count(&self) -> usize {
        0
    }

    fn outputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::Llm)]))
    }

    fn show_input(&mut self, _pin: &InPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
    }

    fn show_output(&mut self, _pin: &OutPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("LLM");
        });

        PinInfo::circle()
            .with_fill(LLM_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn has_body(&self) -> bool {
        true
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
                ui.label("Model");
                let mut text = self.model.borrow().clone();

                if ui.text_edit_singleline(&mut text).changed() {
                    *self.model.borrow_mut() = text;
                }
            });
        });
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(75, 55, 30))
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.model.borrow().clone()),
            _ => None,
        }
    }
}
