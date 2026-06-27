use std::cell::Cell;
use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, NodeCategory, STRING_COLOR};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct ThresholdNode {
    threshold: Cell<u8>,
}

impl ThresholdNode {
    pub fn new() -> Self {
        Self {
            threshold: Cell::new(128),
        }
    }

    pub fn threshold(&self) -> u8 {
        self.threshold.get()
    }
}

impl BaseNode for ThresholdNode {
    fn name(&self) -> &str {
        "Threshold"
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
            ui.label("Threshold:");
            let mut t = self.threshold.get() as i32;
            ui.add(egui::Slider::new(&mut t, 0..=255));
            self.threshold.set(t as u8);
        });
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(60, 40, 70))
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.threshold().to_string()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        if index == 0 && let Ok(v) = value.parse::<u8>() {
            self.threshold.set(v);
        }
    }
}
