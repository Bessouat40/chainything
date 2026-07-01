use std::cell::Cell;
use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, InputOutputType, NodeCategory, NodeInformations, STRING_COLOR,
};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct BrightnessNode {
    delta: Cell<i32>,
}

impl BrightnessNode {
    pub fn new() -> Self {
        Self {
            delta: Cell::new(0),
        }
    }

    pub fn delta(&self) -> i32 {
        self.delta.get()
    }
}

impl BaseNode for BrightnessNode {
    fn name(&self) -> &str {
        "Brightness"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Adjusts image brightness by a signed delta. Positive values brighten, negative values darken; each channel is clamped to 0-255.",
        )
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Image
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        None
    }

    fn is_processor(&self) -> bool {
        true
    }

    fn inputs_count(&self) -> usize {
        1
    }

    fn outputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::RawImage(None))]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::RawImage(None))]))
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.set_min_width(180.0);

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Raw Image");
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
        true
    }

    fn show_body(
        &self,
        _node: egui_snarl::NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _snarl: &egui_snarl::Snarl<Box<dyn BaseNode>>,
    ) {
        ui.horizontal(|ui| {
            ui.label("Delta:");
            let mut d = self.delta.get();
            ui.add(egui::Slider::new(&mut d, -255..=255));
            self.delta.set(d);
        });
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(60, 40, 70))
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.delta().to_string()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        if index == 0
            && let Ok(v) = value.parse::<i32>()
        {
            self.delta.set(v);
        }
    }
}
