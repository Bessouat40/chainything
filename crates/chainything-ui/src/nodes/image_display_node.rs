use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::nodes::base_node::{BaseNode, DisplayData, InputOutputType, NodeCategory, STRING_COLOR};

use chainything::processors::greyscale_processor::RawImage;
use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin, Snarl,
    ui::{PinInfo, WireStyle},
};

/// Hands out a unique id per node instance so each one uploads its texture
/// under a distinct name (egui caches managed textures by name — a shared name
/// would make several image nodes clobber each other on the GPU).
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// Where the displayed image is read from.
#[derive(Clone, Copy, PartialEq)]
enum ImageSource {
    /// The output piped from an upstream node during the last run.
    Node,
    /// A file path or URL typed into the node.
    File,
}

/// Visual *sink* node that displays an image.
///
/// A source selector lets you switch at any time between:
/// - **Node**: the `RawImage` piped from an upstream output during the last run
///   (no need to save to disk), and
/// - **File**: a path / URL typed into the node, refreshable with *Reload*.
#[derive(Clone)]
pub struct ImageDisplayNode {
    id: usize,
    url_input: RefCell<String>,
    /// Which source is currently rendered.
    source: RefCell<ImageSource>,
    /// Image received from an upstream connection during the last run.
    piped: RefCell<Option<RawImage>>,
    /// Cached GPU texture for the piped image.
    texture: RefCell<Option<egui::TextureHandle>>,
    /// Set when a new piped image needs to be (re)uploaded as a texture.
    dirty: Cell<bool>,
}

impl ImageDisplayNode {
    pub fn new() -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            url_input: RefCell::new("".to_string()),
            source: RefCell::new(ImageSource::File),
            piped: RefCell::new(None),
            texture: RefCell::new(None),
            dirty: Cell::new(false),
        }
    }

    fn texture_name(&self) -> String {
        format!("image_display_{}", self.id)
    }
}

/// Builds an egui [`egui::ColorImage`] from a [`RawImage`] (RGB or greyscale).
fn color_image_from_raw(image: &RawImage) -> egui::ColorImage {
    let size = [image.width as usize, image.height as usize];
    let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;

    if is_rgb {
        egui::ColorImage::from_rgb(size, &image.pixels)
    } else {
        // Greyscale: expand each luminance byte to an RGB triple.
        let mut rgb = Vec::with_capacity(image.pixels.len() * 3);
        for &g in &image.pixels {
            rgb.extend_from_slice(&[g, g, g]);
        }
        egui::ColorImage::from_rgb(size, &rgb)
    }
}

impl BaseNode for ImageDisplayNode {
    fn name(&self) -> &str {
        "ImageDisplayNode"
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Image
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
        Some(HashMap::from([(0, InputOutputType::RawImage(None))]))
    }

    fn mapping_output(&self) -> Option<HashMap<usize, InputOutputType>> {
        None
    }

    fn show_input(&mut self, _pin: &InPin, ui: &mut Ui) -> PinInfo {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.label("Raw Image");
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

    fn get_parameter(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.url_input.borrow().clone()),
            1 => Some(match *self.source.borrow() {
                ImageSource::Node => "Node".to_string(),
                ImageSource::File => "File".to_string(),
            }),
            _ => None,
        }
    }

    fn set_parameter(&mut self, index: usize, value: &str) {
        match index {
            0 => *self.url_input.borrow_mut() = value.to_string(),
            1 => {
                *self.source.borrow_mut() = match value {
                    "Node" => ImageSource::Node,
                    _ => ImageSource::File,
                }
            }
            _ => {}
        }
    }

    fn set_display(&self, data: DisplayData) {
        if let DisplayData::Image(image) = data {
            *self.piped.borrow_mut() = Some(image);
            self.dirty.set(true);
            // A fresh run just produced output — show it.
            *self.source.borrow_mut() = ImageSource::Node;
        }
    }

    fn clear_display(&self) {
        *self.piped.borrow_mut() = None;
        *self.texture.borrow_mut() = None;
        self.dirty.set(false);
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

            // Source selector — switch between the piped output and a file/URL.
            let mut source = *self.source.borrow();
            ui.horizontal(|ui| {
                ui.label("Source:");
                ui.radio_value(&mut source, ImageSource::Node, "Node");
                ui.radio_value(&mut source, ImageSource::File, "File / URL");
            });
            *self.source.borrow_mut() = source;

            ui.add_space(4.0);

            match source {
                ImageSource::Node => self.show_piped_image(ui),
                ImageSource::File => self.show_url_image(ui),
            }
        });
    }
}

impl ImageDisplayNode {
    /// Live mode: render the image produced upstream during the last run.
    fn show_piped_image(&self, ui: &mut Ui) {
        if self.dirty.get() {
            if let Some(image) = self.piped.borrow().as_ref() {
                let color = color_image_from_raw(image);
                let handle =
                    ui.ctx()
                        .load_texture(self.texture_name(), color, egui::TextureOptions::LINEAR);
                *self.texture.borrow_mut() = Some(handle);
            }
            self.dirty.set(false);
        }

        let texture = self.texture.borrow();
        if let Some(handle) = texture.as_ref() {
            ui.add(
                egui::Image::new(egui::load::SizedTexture::from_handle(handle))
                    .fit_to_exact_size(egui::Vec2::new(450.0, 450.0)),
            );
        } else {
            ui.allocate_ui(egui::Vec2::new(450.0, 450.0), |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.label("Run the graph to see the output");
                    },
                );
            });
        }
    }

    /// Standalone mode: render an image from a typed path / URL.
    fn show_url_image(&self, ui: &mut Ui) {
        let uri = {
            let local_url = self.url_input.borrow().clone();
            if local_url.is_empty() {
                None
            } else {
                Some(local_url)
            }
        };

        ui.horizontal(|ui| {
            ui.label("URL:");
            let mut text = self.url_input.borrow().clone();
            if ui.text_edit_singleline(&mut text).changed() {
                *self.url_input.borrow_mut() = text;
            }
            // Drop egui's cached copy so an edited file is re-read from disk.
            if ui.button("⟳ Reload").clicked()
                && let Some(uri) = uri.as_ref()
            {
                ui.ctx().forget_image(uri);
            }
        });

        ui.add_space(8.0);

        if let Some(uri) = uri {
            ui.add(
                egui::Image::new(&uri)
                    .show_loading_spinner(true)
                    .fit_to_exact_size(egui::Vec2::new(450.0, 450.0)),
            );
        } else {
            ui.allocate_ui(egui::Vec2::new(450.0, 450.0), |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.label("No image to display");
                    },
                );
            });
        }
    }
}
