use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, InputOutputType, LLM_COLOR, NodeCategory, NodeInformations, STRING_COLOR,
};

use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

/// Pin color for `RawImage` inputs.
const IMAGE_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 200, 100);

/// Visual node wrapping the provider-agnostic `VLMGenerate` processor.
///
/// Takes a *Raw Image*, a text *Prompt* and a *LLM* handle, asks the vision
/// language model to describe or reason about the image, and outputs the
/// generated *Response* (`String`). Any vision-capable loader (e.g.
/// `OllamaLoader`) can be connected to the `LLM` input interchangeably.
#[derive(Clone)]
pub struct VlmGenerateNode;

impl VlmGenerateNode {
    pub fn new() -> Self {
        Self
    }
}

impl BaseNode for VlmGenerateNode {
    fn name(&self) -> &str {
        "VLMGenerate"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Generates text from an image and a prompt using a loaded vision language model. Connect any LLM loader to the LLM input.",
        )
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
        3
    }

    fn outputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([
            (0, InputOutputType::RawImage(None)),
            (1, InputOutputType::String("".to_string())),
            (2, InputOutputType::Llm),
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
                    ui.label("Raw Image");
                });
                PinInfo::circle()
                    .with_fill(IMAGE_COLOR)
                    .with_wire_style(WireStyle::AxisAligned {
                        corner_radius: 10.0,
                    })
            }
            1 => {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label("String");
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
            ui.label("String");
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
