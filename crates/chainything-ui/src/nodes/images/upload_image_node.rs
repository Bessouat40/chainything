use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{
    nodes::base_node::{
        BaseNode, InputOutputType, NodeCategory, NodeInformations, STRING_COLOR,
    },
};

use chainything::processors::images::greyscale_processor::RawImage;

use egui::Ui;
use egui_snarl::{
    InPin, NodeId, OutPin,
    ui::{PinInfo, WireStyle},
};

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
enum UploadResult {
    Loaded {
        filename: String,
        image: RawImage,
    },
    Cancelled,
    Error(String),
}

#[derive(Clone)]
pub struct ImageUploadNode {
    output: Vec<InputOutputType>,
    filename: Option<String>,
    dimensions: Option<(u32, u32)>,
    error: Option<String>,
    loading: bool,

    #[cfg(target_arch = "wasm32")]
    pending_result: Rc<RefCell<Option<UploadResult>>>,
}

impl ImageUploadNode {
    pub fn new() -> Self {
        Self {
            output: vec![InputOutputType::RawImage(None)],
            filename: None,
            dimensions: None,
            error: None,
            loading: false,

            #[cfg(target_arch = "wasm32")]
            pending_result: Rc::new(RefCell::new(None)),
        }
    }

    fn decode_image(bytes: &[u8]) -> Result<RawImage, String> {
        let image = image::load_from_memory(bytes)
            .map_err(|err| format!("Failed to decode image: {err}"))?;

        // On normalise tout en RGB8.
        let rgb = image.to_rgb8();

        let width = rgb.width();
        let height = rgb.height();

        Ok(RawImage {
            width,
            height,
            pixels: rgb.into_raw(),
        })
    }

    fn set_image(&mut self, filename: String, image: RawImage) {
        self.dimensions = Some((image.width, image.height));
        self.filename = Some(filename);
        self.error = None;
        self.loading = false;

        self.output[0] = InputOutputType::RawImage(Some(image));
    }

    #[cfg(target_arch = "wasm32")]
    fn poll_upload_result(&mut self) {
        let result = self.pending_result.borrow_mut().take();

        let Some(result) = result else {
            return;
        };

        match result {
            UploadResult::Loaded { filename, image } => {
                self.set_image(filename, image);
            }

            UploadResult::Cancelled => {
                self.loading = false;
            }

            UploadResult::Error(error) => {
                self.loading = false;
                self.error = Some(error);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn open_file_picker(&mut self, ctx: egui::Context) {
        self.loading = true;
        self.error = None;

        let pending_result = self.pending_result.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter(
                    "Images",
                    &["png", "jpg", "jpeg", "bmp", "webp"],
                )
                .pick_file()
                .await;

            let result = match file {
                Some(file) => {
                    let filename = file.file_name();

                    let bytes = file.read().await;

                    match Self::decode_image(&bytes) {
                        Ok(image) => UploadResult::Loaded {
                            filename,
                            image,
                        },

                        Err(error) => UploadResult::Error(error),
                    }
                }

                None => UploadResult::Cancelled,
            };

            *pending_result.borrow_mut() = Some(result);

            ctx.request_repaint();
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_file_picker(&mut self) {
        self.loading = true;
        self.error = None;

        let path = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "bmp", "webp"],
            )
            .pick_file();

        let Some(path) = path else {
            self.loading = false;
            return;
        };

        let filename = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "image".to_string());

        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,

            Err(err) => {
                self.loading = false;
                self.error = Some(format!("Failed to read image: {err}"));
                return;
            }
        };

        match Self::decode_image(&bytes) {
            Ok(image) => {
                self.set_image(filename, image);
            }

            Err(error) => {
                self.loading = false;
                self.error = Some(error);
            }
        }
    }
}

impl Default for ImageUploadNode {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseNode for ImageUploadNode {
    fn name(&self) -> &str {
        "ImageUpload"
    }

    fn informations(&self) -> NodeInformations {
        NodeInformations::new(
            "Uploads an image from the user's computer and outputs it as raw image data.",
        )
    }

    fn category(&self) -> NodeCategory {
        NodeCategory::Image
    }

    fn is_processor(&self) -> bool {
        false
    }

    fn get_value(&self) -> Option<&Vec<InputOutputType>> {
        Some(&self.output)
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
        Some(HashMap::from([(
            0,
            InputOutputType::RawImage(None),
        )]))
    }

    fn show_input(
        &mut self,
        _pin: &InPin,
        _ui: &mut Ui,
    ) -> PinInfo {
        PinInfo::circle()
    }

    fn show_output(
        &mut self,
        _pin: &OutPin,
        ui: &mut Ui,
    ) -> PinInfo {
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                ui.label("Raw Image");
            },
        );

        PinInfo::circle()
            .with_fill(STRING_COLOR)
            .with_wire_style(WireStyle::AxisAligned {
                corner_radius: 10.0,
            })
    }

    fn has_body(&self) -> bool {
        true
    }

    fn header_frame(
        &self,
        frame: egui::Frame,
    ) -> egui::Frame {
        frame.fill(
            egui::Color32::from_rgb(70, 40, 40),
        )
    }

    fn show_body(
        &mut self,
        _node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
    ) {
        #[cfg(target_arch = "wasm32")]
        self.poll_upload_result();

        ui.with_layout(
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(200.0);

                let button = ui.add_enabled(
                    !self.loading,
                    egui::Button::new("Choose image"),
                );

                if button.clicked() {
                    #[cfg(target_arch = "wasm32")]
                    {
                        self.open_file_picker(ui.ctx().clone());
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        self.open_file_picker();
                    }
                }

                if self.loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Loading...");
                    });
                }

                if let Some(filename) = &self.filename {
                    ui.label(format!("File: {filename}"));
                }

                if let Some((width, height)) = self.dimensions {
                    ui.label(format!("{width} x {height}"));
                }

                if let Some(error) = &self.error {
                    ui.colored_label(
                        egui::Color32::RED,
                        error,
                    );
                }
            },
        );
    }

    fn get_parameter(
        &self,
        _index: usize,
    ) -> Option<String> {
        None
    }

    fn set_parameter(
        &mut self,
        _index: usize,
        _value: &str,
    ) {
    }
}