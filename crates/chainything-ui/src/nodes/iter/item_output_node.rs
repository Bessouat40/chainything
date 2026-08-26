use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, ITEM_COLOR, InputOutputType, NodeCategory, NodeInformations,
};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

/// Terminal marker of a `ForEach` loop body: whatever is wired here becomes the
/// per-element result.
///
/// It maps to the backend `ItemOutput` passthrough, which `ForEach` reads as the
/// result of each iteration. Its single input is an [`InputOutputType::Item`]
/// wildcard, so any node's output can feed it.
#[derive(Clone)]
pub struct ItemOutputNode;

impl ItemOutputNode {
    pub fn new() -> Self {
        Self
    }
}

impl BaseNode for ItemOutputNode {
    fn name(&self) -> &str {
        "ItemOutput"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Loop-body terminal: whatever you connect here becomes the result for the current \
             element. The ForEach node collects one such result per element.",
        )
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Flow
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
        0
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::Item)]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.set_min_width(140.0);
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Result");
        });

        PinInfo::circle()
            .with_fill(ITEM_COLOR)
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
        frame.fill(egui::Color32::from_rgb(90, 60, 100))
    }
}
