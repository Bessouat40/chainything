use std::collections::HashMap;
use std::cell::RefCell;

use crate::nodes::base_node::{BaseNode, InputOutputType};

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo},
};

#[derive(Clone)]
pub struct ImageDisplayNode {
    url_input: RefCell<String>,
}

impl ImageDisplayNode {
    pub fn new() -> Self {
        Self {
            url_input: RefCell::new("".to_string()),
        }
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
        0
    }

    fn outputs_count(&self) -> usize {
        0
    }

    fn mapping_input(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn show_input(&mut self, _pin: &InPin, _ui: &mut Ui) -> PinInfo {
        PinInfo::circle()
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
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _snarl: &Snarl<Box<dyn BaseNode>>,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_width(500.0);
            ui.set_height(500.0);

            ui.add_enabled_ui(true, |ui| {
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    let mut text = self.url_input.borrow().clone();
                
                    if ui.text_edit_singleline(&mut text).changed() {
                        *self.url_input.borrow_mut() = text;
                    }
                });
            });

            ui.add_space(8.0);

            let final_uri = {
                let local_url = self.url_input.borrow().clone();
                if local_url.is_empty() { None } else { Some(local_url) }
            };

            if let Some(uri) = final_uri {
                ui.add(
                    egui::Image::new(&uri)
                        .show_loading_spinner(true)
                        .fit_to_exact_size(egui::Vec2::new(450.0, 450.0)),
                );
            } else {
                ui.allocate_ui(egui::Vec2::new(450.0, 450.0), |ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| { ui.label("No image to display"); }
                    );
                });
            }
        });
    }
}