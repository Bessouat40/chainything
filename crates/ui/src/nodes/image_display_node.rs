use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType, STRING_COLOR};

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

#[derive(Clone)]
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

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        None
    }

    fn is_processor(&self) -> bool {
        false
    }

    fn inputs_count(&self) -> usize {
        1
    }

    fn outputs_count(&self) -> usize {
        0
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        Some(HashMap::from([(
            0,
            InputOutputType::String("".to_string()),
        )]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
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

    fn show_output(&mut self, _pin: &OutPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
    }

    fn has_body(&self) -> bool {
        true
    }

    fn header_frame(&self, frame: egui::Frame) -> egui::Frame {
        frame.fill(egui::Color32::from_rgb(70, 40, 40))
    }

    fn show_body(
        &self,
        _node: NodeId,
        inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &Snarl<Box<dyn BaseNode>>,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_width(200.0);

            let input = &inputs[0];
            let url_to_display = match input.remotes.as_slice() {
                [remote] => {
                    let remote_node = &snarl[remote.node];

                    if let Some(values) = remote_node.get_value() {
                        if let Some(InputOutputType::String(uri)) = values.get(remote.output) {
                            Some(uri.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(uri) = url_to_display {
                ui.add(egui::Image::new(&uri).show_loading_spinner(true));
            } else {
                ui.label("No image to display");
            }
        });
    }
}
