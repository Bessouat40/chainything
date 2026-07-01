use std::cell::RefCell;
use std::collections::HashMap;

use crate::nodes::base_node::{
    BaseNode, DisplayData, InputOutputType, NodeCategory, NodeInformations, STRING_COLOR,
};

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

/// Visual *sink* node that displays text.
///
/// Two ways to feed it:
/// - **Connected**: wire any text output (e.g. `LLMGenerate`) into its input
///   pin and the generated text appears after a run — no need to save to disk.
/// - **Standalone**: leave it unconnected and type a file path to view its
///   contents (re-read when the path changes or on *Reload*).
#[derive(Clone)]
pub struct TextDisplayNode {
    path_input: RefCell<String>,
    loaded_path: RefCell<String>,
    content: RefCell<String>,
    /// Text received from an upstream connection during the last run.
    piped: RefCell<Option<String>>,
}

impl TextDisplayNode {
    pub fn new() -> Self {
        Self {
            path_input: RefCell::new("".to_string()),
            loaded_path: RefCell::new("\0".to_string()),
            content: RefCell::new("".to_string()),
            piped: RefCell::new(None),
        }
    }
}

impl BaseNode for TextDisplayNode {
    fn name(&self) -> &str {
        "TextDisplayNode"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Displays text in the graph: either piped from an upstream connection after a run, or read from a file path when left unconnected.",
        )
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Display
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
            ui.label("Text");
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
        frame.fill(egui::Color32::from_rgb(40, 55, 40))
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
            // Force a re-read of the file on next render in standalone mode.
            *self.loaded_path.borrow_mut() = "\0".to_string();
        }
    }

    fn set_display(&self, data: DisplayData) {
        if let DisplayData::Text(text) = data {
            *self.piped.borrow_mut() = Some(text);
        }
    }

    fn clear_display(&self) {
        *self.piped.borrow_mut() = None;
    }

    fn show_body(
        &self,
        _node: NodeId,
        inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _snarl: &Snarl<Box<dyn BaseNode>>,
    ) {
        let connected = inputs
            .first()
            .map(|pin| !pin.remotes.is_empty())
            .unwrap_or(false);

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(400.0);

            if connected {
                // Live mode: show whatever flowed into the pin during the last run.
                let text = self.piped.borrow().clone();
                match text {
                    Some(text) => self.show_text(ui, &text),
                    None => self.show_placeholder(ui, "Run the graph to see the output"),
                }
            } else {
                // Standalone mode: read from a file path.
                ui.horizontal(|ui| {
                    ui.label("File:");
                    let mut text = self.path_input.borrow().clone();
                    if ui.text_edit_singleline(&mut text).changed() {
                        *self.path_input.borrow_mut() = text;
                    }
                    if ui.button("⟳ Reload").clicked() {
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
                    self.show_placeholder(ui, "No file to display");
                } else {
                    let content = self.content.borrow().clone();
                    self.show_text(ui, &content);
                }
            }
        });
    }
}

impl TextDisplayNode {
    fn show_text(&self, ui: &mut Ui, text: &str) {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut buffer = text.to_string();
                ui.add(
                    egui::TextEdit::multiline(&mut buffer)
                        .desired_width(f32::INFINITY)
                        .desired_rows(10)
                        .font(egui::TextStyle::Monospace)
                        .interactive(false),
                );
            });
    }

    fn show_placeholder(&self, ui: &mut Ui, message: &str) {
        ui.allocate_ui(egui::Vec2::new(400.0, 120.0), |ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.label(message);
                },
            );
        });
    }
}
