use std::cell::Cell;
use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, InputOutputType, MESH_COLOR, NodeCategory, NodeInformations, STRING_COLOR,
};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
pub struct ModelRenderNode {
    resolution: Cell<u32>,
}

impl ModelRenderNode {
    pub fn new() -> Self {
        Self {
            resolution: Cell::new(512),
        }
    }

    pub fn resolution(&self) -> u32 {
        self.resolution.get()
    }
}

impl BaseNode for ModelRenderNode {
    fn name(&self) -> &str {
        "ModelRender"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Renders a 3D mesh to a 2D raw image (three-quarter view, shaded), \
             ready to display or save.",
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
        Some(HashMap::from([(0, InputOutputType::RawImage(None))]))
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
            ui.label("Resolution:");
            let mut r = self.resolution.get();
            ui.add(egui::DragValue::new(&mut r).speed(8.0).range(1..=4096));
            self.resolution.set(r);
        });
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(50, 55, 80))
    }

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.resolution().to_string()),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        if index == 0
            && let Ok(v) = value.parse::<u32>()
        {
            self.resolution.set(v);
        }
    }
}
