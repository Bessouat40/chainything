use std::cell::RefCell;
use std::collections::HashMap;

use crate::nodes::base_node::{BaseNode, InputOutputType};

use egui::Ui;
use egui_snarl::{InPin, NodeId, OutPin, Snarl, ui::PinInfo};

/// Visual *sink* node that displays the contents of a text file.
///
/// Like [`ImageDisplayNode`](crate::nodes::image_display_node::ImageDisplayNode),
/// this is not a processor: it reads from a path you type in (for example the
/// file written by a `TextSave` node) and shows its content read-only. The file
/// is re-read when the path changes or when *Reload* is clicked.
#[derive(Clone)]
pub struct TextDisplayNode {
    path_input: RefCell<String>,
    loaded_path: RefCell<String>,
    content: RefCell<String>,
}

impl TextDisplayNode {
    pub fn new() -> Self {
        Self {
            path_input: RefCell::new("".to_string()),
            loaded_path: RefCell::new("\0".to_string()),
            content: RefCell::new("".to_string()),
        }
    }
}

impl BaseNode for TextDisplayNode {
    fn name(&self) -> &str {
        "TextDisplayNode"
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
        frame.fill(egui::Color32::from_rgb(40, 55, 40))
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
            ui.set_width(400.0);

            ui.horizontal(|ui| {
                ui.label("File:");
                let mut text = self.path_input.borrow().clone();
                if ui.text_edit_singleline(&mut text).changed() {
                    *self.path_input.borrow_mut() = text;
                }
                if ui.button("⟳ Reload").clicked() {
                    // Force a re-read on the next refresh.
                    *self.loaded_path.borrow_mut() = "\0".to_string();
                }
            });

            ui.add_space(6.0);

            let path = self.path_input.borrow().clone();
            if *self.loaded_path.borrow() != path {
                let loaded = if path.is_empty() {
                    String::new()
                } else {
                    std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| format!("⚠ Unable to read file: {}", e))
                };
                *self.content.borrow_mut() = loaded;
                *self.loaded_path.borrow_mut() = path.clone();
            }

            if path.is_empty() {
                ui.allocate_ui(egui::Vec2::new(400.0, 200.0), |ui| {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.label("No file to display");
                        },
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut content = self.content.borrow().clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut content)
                                .desired_width(f32::INFINITY)
                                .desired_rows(10)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false),
                        );
                    });
            }
        });
    }
}
