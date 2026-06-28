use std::cell::RefCell;
use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, InputOutputType, MESH_COLOR, NodeCategory, NodeInformations,
};

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct ModelSaveNode {
    path_input: RefCell<String>,
}

impl ModelSaveNode {
    pub fn new() -> Self {
        Self {
            path_input: RefCell::new("".to_string()),
        }
    }
}

impl BaseNode for ModelSaveNode {
    fn name(&self) -> &str {
        "ModelSaveNode"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Saves the incoming 3D mesh to disk as a Wavefront OBJ file at the configured path.",
        )
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Model3D
    }

    fn is_processor(&self) -> bool {
        true
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        None
    }

    fn outputs_count(&self) -> usize {
        0
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn inputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::Mesh3D(None))]))
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Mesh");
            ui.add_space(5.0);
        });

        PinInfo::circle()
            .with_fill(MESH_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn show_output(&mut self, _pin: &OutPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
    }

    fn has_body(&self) -> bool {
        true
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(40, 70, 75))
    }

    fn show_body(
        &self,
        _node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _snarl: &Snarl<Box<dyn BaseNode>>,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(200.0);

            ui.horizontal(|ui| {
                ui.label("Save To");
                let mut text = self.path_input.borrow().clone();

                if ui.text_edit_singleline(&mut text).changed() {
                    *self.path_input.borrow_mut() = text;
                }
            });
        });
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.path_input.borrow().clone()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        if index == 0 {
            *self.path_input.borrow_mut() = value.to_string();
        }
    }
}
