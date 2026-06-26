use std::collections::HashMap;
use std::cell::Cell;

use crate::nodes::base_node::{BaseNode, InputOutputType, STRING_COLOR};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct ResizeNode {
    width: Cell<u32>,
    height: Cell<u32>,
}

impl ResizeNode {
    pub fn new() -> Self {
        Self {
            width: Cell::new(256),
            height: Cell::new(256),
        }
    }

    pub fn width(&self) -> u32 {
        self.width.get()
    }

    pub fn height(&self) -> u32 {
        self.height.get()
    }
}

impl BaseNode for ResizeNode {
    fn name(&self) -> &str {
        "Resize"
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
            ui.label("W:");
            let mut w = self.width.get() as i32;
            ui.add(egui::DragValue::new(&mut w).range(1..=4096));
            self.width.set(w as u32);
        });
        ui.horizontal(|ui| {
            ui.label("H:");
            let mut h = self.height.get() as i32;
            ui.add(egui::DragValue::new(&mut h).range(1..=4096));
            self.height.set(h as u32);
        });
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(60, 40, 70))
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.width().to_string()),
            1 => Some(self.height().to_string()),
            _ => None,
        }
    }
}
