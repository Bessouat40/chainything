use std::cell::Cell;
use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, InputOutputType, MESH_COLOR, NodeCategory, NodeInformations,
};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct ModelScaleNode {
    factor: Cell<f32>,
}

impl ModelScaleNode {
    pub fn new() -> Self {
        Self {
            factor: Cell::new(1.0),
        }
    }

    pub fn factor(&self) -> f32 {
        self.factor.get()
    }
}

impl BaseNode for ModelScaleNode {
    fn name(&self) -> &str {
        "ModelScale"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Uniformly scales a 3D mesh about the origin by the configured factor.",
        )
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Model3D
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
        Some(HashMap::from([(0, InputOutputType::Mesh3D(None))]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::Mesh3D(None))]))
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.set_min_width(180.0);

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Mesh");
        });

        PinInfo::circle()
            .with_fill(MESH_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn show_output(&mut self, _pin: &OutPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("Mesh");
        });

        PinInfo::circle()
            .with_fill(MESH_COLOR)
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
            ui.label("Factor:");
            let mut f = self.factor.get();
            ui.add(egui::DragValue::new(&mut f).speed(0.05).range(0.0..=1000.0));
            self.factor.set(f);
        });
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(45, 60, 80))
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.factor().to_string()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        if index == 0
            && let Ok(v) = value.parse::<f32>()
        {
            self.factor.set(v);
        }
    }
}
