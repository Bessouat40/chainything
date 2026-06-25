use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, STRING_COLOR};

use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct TextInputNode {
    value: Vec<InputOutputType>,
}

impl TextInputNode {
    pub fn new() -> Self {
        Self {
            value: vec![InputOutputType::String("".to_string())],
        }
    }
}

impl BaseNode for TextInputNode {
    fn name(&self) -> &str {
        "TextInputNode"
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        Some(&self.value)
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
        Some(HashMap::from([(
            0,
            InputOutputType::String("".to_string()),
        )]))
    }

    fn show_input(&mut self, _pin: &InPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
    }

    fn show_output(&mut self, _pin: &OutPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
            ui.label("String");

            if let InputOutputType::String(ref mut val) = self.value[0] {
                ui.add(
                    egui::TextEdit::singleline(val)
                        .text_color(STRING_COLOR)
                        .desired_width(120.0)
                        .font(egui::TextStyle::Monospace),
                );
            }
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
        frame.fill(egui::Color32::from_rgb(70, 70, 80))
    }
}
