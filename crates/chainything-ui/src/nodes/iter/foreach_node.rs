use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, InputOutputType, LIST_COLOR, NodeCategory, NodeInformations,
};
use crate::nodes::iter::{item_input_node::ItemInputNode, item_output_node::ItemOutputNode};
use egui::Ui;
use egui_snarl::{
    InPin, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

/// Applies a sub-pipeline (its "loop body") to every element of an input list.
///
/// The loop body is a nested [`Snarl`] edited by drilling into this node (see
/// [`DAGLayout`](crate::dag_layout::DAGLayout)). It starts scaffolded with an
/// [`ItemInputNode`] (the element source) and an [`ItemOutputNode`] (the element
/// result). At export time the editor turns this nested graph into the backend
/// `ForEach` node's `sub_pipeline`, so any existing processor is reused as-is
/// inside the loop.
///
/// # Pins
/// - input `0`: the collection to iterate ([`InputOutputType::List`]).
/// - output `0`: the list of per-element results.
/// - output `1`: the list of per-element errors (partial-failure report).
#[derive(Clone)]
pub struct ForEachNode {
    /// The loop body, edited as its own graph. Reused across frames.
    body: Snarl<Box<dyn BaseNode>>,
}

impl ForEachNode {
    pub fn new() -> Self {
        // Scaffold the body with the two markers so the loop is valid out of the
        // box: the user only has to drop processors between them.
        let mut body: Snarl<Box<dyn BaseNode>> = Snarl::new();
        body.insert_node(
            egui::pos2(40.0, 100.0),
            Box::new(ItemInputNode::new()) as Box<dyn BaseNode>,
        );
        body.insert_node(
            egui::pos2(340.0, 100.0),
            Box::new(ItemOutputNode::new()) as Box<dyn BaseNode>,
        );
        Self { body }
    }
}

impl BaseNode for ForEachNode {
    fn name(&self) -> &str {
        "ForEach"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Runs its loop body once per element of the incoming list, in parallel. \
             Right-click and pick \"Edit loop body\" to build the sub-pipeline. \
             Outputs the list of results and the list of per-element errors.",
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
        2
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::List)]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([
            (0, InputOutputType::List),
            (1, InputOutputType::List),
        ]))
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.set_min_width(160.0);
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Collection");
        });

        PinInfo::circle()
            .with_fill(LIST_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui) -> PinInfo {
        let label = if pin.id.output == 0 { "Results" } else { "Errors" };
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(label);
        });

        PinInfo::circle()
            .with_fill(LIST_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn has_body(&self) -> bool {
        false
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(110, 70, 130))
    }

    fn has_sub_editor(&self) -> bool {
        true
    }

    fn sub_snarl(&self) -> Option<&Snarl<Box<dyn BaseNode>>> {
        Some(&self.body)
    }

    fn sub_snarl_mut(&mut self) -> Option<&mut Snarl<Box<dyn BaseNode>>> {
        Some(&mut self.body)
    }
}
