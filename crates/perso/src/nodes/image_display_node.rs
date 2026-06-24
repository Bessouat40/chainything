use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, STRING_COLOR};

use egui::{Color32, Image, Ui};
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{
        AnyPins, PinInfo, SnarlViewer,
        WireStyle,
    },
};


pub struct ImageDisplayNode;

impl ImageDisplayNode {
    pub fn new() -> Self {
        Self
    }
}

impl BaseNode for ImageDisplayNode {
    fn name(&self) -> &str {
        "ImageDisplayNode"
    }

    fn inputs_count(&self) -> usize {
        1
    }

    fn outputs_count(&self) -> usize {
        0
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(0, InputOutputType::RawImage(None))]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Input String");
            
            ui.add_space(5.0);
        });

        PinInfo::circle()
            .with_fill(STRING_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui) -> PinInfo {
        unreachable!("ImageDisplay node has no outputs")
    }

    fn has_body(&self) -> bool {
        true
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(70, 40, 40))
    }

    // fn show_body(
    //     &mut self,
    //     node: NodeId,
    //     inputs: &[InPin],
    //     _outputs: &[OutPin],
    //     ui: &mut Ui,
    //     snarl: &mut Snarl<Box<dyn BaseNode>>,
    // ) {
    //     ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    //     ui.set_width(200.0);
    //     let input = &inputs[0];
    //     let url_to_display = match input.remotes.as_slice() {
    //         [remote] => {
    //                 Some(snarl[remote.node].string_in().clone())
    //             }
    //             _ => None,
    //         };

    //         if let Some(uri) = url_to_display {
    //             ui.add(
    //                 egui::Image::new(&uri)
    //                     .show_loading_spinner(true)
    //             );
    //         } else {
    //             ui.label("No image to display");
    //         }
    //     });
    // }
}