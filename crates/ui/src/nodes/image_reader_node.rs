use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, STRING_COLOR};

use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct ImageReaderNode;

impl ImageReaderNode {
    pub fn new() -> Self {
        Self
    }
}

impl BaseNode for ImageReaderNode {
    fn name(&self) -> &str {
        "ImageReader"
    }

    fn is_processor(&self) -> bool {
        true
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        None
    }

    fn inputs_count(&self) -> usize {
        1
    }

    fn outputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(
            0,
            InputOutputType::String("".to_string()),
        )]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::RawImage(None))]))
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.set_min_width(200.0); 

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("String");
        });

        PinInfo::circle()
            .with_fill(STRING_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn show_output(&mut self, _pin: &OutPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("Raw Image");
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
        frame.fill(egui::Color32::from_rgb(70, 40, 40))
    }
}
