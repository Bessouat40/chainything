use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, ITEM_COLOR, InputOutputType, NodeCategory, NodeInformations,
};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin,
    ui::{PinInfo, WireStyle},
};

/// Entry marker of a `ForEach` loop body: emits the current element.
///
/// It maps to the backend `ItemInput` passthrough. Its single output is an
/// [`InputOutputType::Item`] wildcard, so it can feed any node inside the loop
/// regardless of the collection's element type.
#[derive(Clone)]
pub struct ItemInputNode;

impl ItemInputNode {
    pub fn new() -> Self {
        Self
    }
}

impl BaseNode for ItemInputNode {
    fn name(&self) -> &str {
        "ItemInput"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Loop-body entry: emits the current element of the collection the ForEach iterates. \
             Place downstream nodes after it, then feed the result into ItemOutput.",
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
        0
    }

    fn outputs_count(&self) -> usize {
        1
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::Item)]))
    }

    fn show_input(&mut self, _pin: &InPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
    }

    fn show_output(&mut self, _pin: &OutPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label("Element");
        });

        PinInfo::circle()
            .with_fill(ITEM_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn has_body(&self) -> bool {
        false
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(90, 60, 100))
    }
}
