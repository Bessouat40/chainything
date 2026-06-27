use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, LLM_COLOR, NodeCategory, STRING_COLOR};

use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

/// Visual node wrapping the provider-agnostic `LLMGenerate` processor.
///
/// Takes a text *Prompt* (`String`) and a *LLM* handle, calls the model and
/// outputs the generated *Response* (`String`). Any loader (Ollama, OpenAI,
/// Mistral, Claude, ...) can be connected to the `LLM` input interchangeably.
#[derive(Clone)]
pub struct LlmGenerateNode;

impl LlmGenerateNode {
    pub fn new() -> Self {
        Self
    }
}

impl BaseNode for LlmGenerateNode {
    fn name(&self) -> &str {
        "LLMGenerate"
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
        2
    }

    fn outputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([
            (0, InputOutputType::String("".to_string())),
            (1, InputOutputType::Llm),
        ]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(
            0,
            InputOutputType::String("".to_string()),
        )]))
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.set_min_width(180.0);

        match pin.id.input {
            0 => {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label("Prompt");
                });
                PinInfo::circle()
                    .with_fill(STRING_COLOR)
                    .with_wire_style(WireStyle::AxisAligned {
                        corner_radius: 10.0,
                    })
            }
            _ => {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label("LLM");
                });
                PinInfo::circle()
                    .with_fill(LLM_COLOR)
                    .with_wire_style(WireStyle::AxisAligned {
                        corner_radius: 10.0,
                    })
            }
        }
    }

    fn show_output(&mut self, _pin: &OutPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("Response");
        });

        PinInfo::circle()
            .with_fill(STRING_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn has_body(&self) -> bool {
        false
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(40, 55, 75))
    }
}
