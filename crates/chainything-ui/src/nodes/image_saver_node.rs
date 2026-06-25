use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, STRING_COLOR};

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct ImageSaveNode;

impl ImageSaveNode {
    pub fn new() -> Self {
        Self
    }
}

impl BaseNode for ImageSaveNode {
    fn name(&self) -> &str {
        "ImageSaveNode"
    }

    fn is_processor(&self) -> bool {
        true
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        None
    }

    fn inputs_count(&self) -> usize {
        2
    }

    fn outputs_count(&self) -> usize {
        0
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([
            (0, InputOutputType::RawImage(None)),
            (1, InputOutputType::String(String::new())),
        ]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if pin.id.input == 0 {
                ui.label("Raw Image");
            } else if pin.id.input == 1 {
                ui.label("String");
            }

            ui.add_space(5.0);
        });

        let pin_color = if pin.id.input == 0 {
            egui::Color32::from_rgb(100, 200, 100)
        } else {
            STRING_COLOR
        };

        PinInfo::circle()
            .with_fill(pin_color)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn show_output(&mut self, _pin: &OutPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
    }

    fn has_body(&self) -> bool {
        false
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(40, 70, 40))
    }

    fn show_body(
        &self,
        _node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        _ui: &mut Ui,
        _snarl: &Snarl<Box<dyn BaseNode>>,
    ) {
    }
}
