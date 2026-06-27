use std::cell::Cell;
use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, NodeCategory, STRING_COLOR};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct BlurNode {
    radius: Cell<u32>,
}

impl BlurNode {
    pub fn new() -> Self {
        Self {
            radius: Cell::new(1),
        }
    }

    pub fn radius(&self) -> u32 {
        self.radius.get()
    }
}

impl BaseNode for BlurNode {
    fn name(&self) -> &str {
        "Blur"
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
            ui.label("Radius:");
            let mut r = self.radius.get() as i32;
            ui.add(egui::Slider::new(&mut r, 1..=10));
            self.radius.set(r as u32);
        });
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(60, 40, 70))
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.radius().to_string()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        if index == 0 && let Ok(v) = value.parse::<u32>() {
            self.radius.set(v);
        }
    }
}
