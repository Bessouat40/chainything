use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, STRING_COLOR};

use egui::{Color32, Ui};
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{
        AnyPins, PinInfo, SnarlViewer,
        WireStyle,
    },
};

#[derive(Clone)]
pub struct TextInputNode{
    value: String,
}

impl TextInputNode {
    pub fn new() -> Self {
        Self {
            value: String::new(),
        }
    }
}

impl BaseNode for TextInputNode {
    fn name(&self) -> &str {
        "TextInputNode"
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
        Some(HashMap::from([(0, InputOutputType::String("".to_string()))]))
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut Ui) -> PinInfo {
        unreachable!("TextInputProcessor node has no inputs")

    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui) -> PinInfo {
        assert_eq!(pin.id.output, 0, "TextInputProcessor node has only one output");

        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
            ui.label("Output String:");
            
            let edit = egui::TextEdit::singleline(&mut self.value)
                .text_color(STRING_COLOR)
                .desired_width(120.0) 
                .font(egui::TextStyle::Monospace);
            
            ui.add(edit);
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