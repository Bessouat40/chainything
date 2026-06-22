use egui::*;
use std::path::PathBuf;
use crate::node_editor::NodeEditor;
use crate::panel_left::PanelLeft;
use crate::types::{load_processor_defs, ProcessorDef};
use chainything::prelude::*;

pub struct ChainythingApp {
    panel_left: PanelLeft,
    node_editor: NodeEditor,
    dragging_processor: Option<ProcessorDef>,
    status: Status,
    json_preview: String,
    show_json: bool,
}

enum Status {
    Idle,
    Success,
    Error(String),
}

impl ChainythingApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Look for nodes.toml next to the binary, then in the current dir
        let nodes_path = Self::find_nodes_toml();
        let defs = load_processor_defs(&nodes_path);

        Self {
            panel_left: PanelLeft::new(defs.clone()),
            node_editor: NodeEditor::new(defs),
            dragging_processor: None,
            status: Status::Idle,
            json_preview: String::new(),
            show_json: false,
        }
    }

    fn find_nodes_toml() -> PathBuf {
        // 1. next to the binary
        if let Ok(mut p) = std::env::current_exe() {
            p.pop();
            p.push("nodes.toml");
            if p.exists() { return p; }
        }
        // 2. current working directory
        PathBuf::from("nodes.toml")
    }
}

impl eframe::App for ChainythingApp {
    fn ui(&mut self, ctx: &mut egui::Ui, _frame: &mut eframe::Frame) {
        apply_theme(ctx);

        // ── Left panel ───────────────────────────────────────────────────────
        let to_add = self.panel_left.show(ctx, &mut self.dragging_processor);
        if let Some(type_name) = to_add {
            self.node_editor.add_node(&type_name, Pos2::new(200.0, 150.0));
        }

        // ── Bottom bar ───────────────────────────────────────────────────────
        egui::Panel::bottom("bottom_bar")
            .exact_size(48.0)
            .frame(Frame {
                fill: Color32::from_rgb(14, 14, 22),
                inner_margin: Margin::symmetric(16, 0),
                ..Default::default()
            })
            .show_inside(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let (status_text, status_color) = match &self.status {
                        Status::Idle => ("Ready", Color32::from_rgb(100, 100, 130)),
                        Status::Success => ("✓ Pipeline executed successfully!", Color32::from_rgb(80, 210, 140)),
                        Status::Error(e) => (e.as_str(), Color32::from_rgb(220, 80, 80)),
                    };
                    ui.label(RichText::new(status_text).size(12.0).color(status_color));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let exec_btn = ui.add(
                            Button::new(RichText::new("▶  Execute").size(13.0).color(Color32::WHITE).strong())
                                .fill(Color32::from_rgb(60, 120, 220))
                                .corner_radius(6u8)
                                .min_size(Vec2::new(120.0, 32.0)),
                        );
                        if exec_btn.clicked() {
                            self.execute_pipeline();
                        }

                        ui.add_space(8.0);

                        let json_btn = ui.add(
                            Button::new(RichText::new("{ }").size(13.0).color(Color32::from_rgb(180, 180, 220)))
                                .fill(Color32::from_rgb(30, 30, 50))
                                .corner_radius(6u8)
                                .min_size(Vec2::new(40.0, 32.0)),
                        );
                        if json_btn.clicked() {
                            self.json_preview = self.node_editor.to_json().unwrap_or_default();
                            self.show_json = !self.show_json;
                        }
                    });
                });
            });

        // ── Central canvas ───────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show_inside(ctx, |ui| {
                let egui_ctx = ui.ctx().clone();
                self.node_editor.show(&egui_ctx, ui);

                if let Some(proc) = self.dragging_processor.take() {
                    if egui_ctx.input(|i| i.pointer.primary_released()) {
                        if let Some(pos) = egui_ctx.input(|i| i.pointer.hover_pos()) {
                            self.node_editor.add_node(&proc.type_name, pos);
                        }
                    } else {
                        self.dragging_processor = Some(proc);
                    }
                }
            });

        // ── JSON preview window ──────────────────────────────────────────────
        if self.show_json {
            let mut open = self.show_json;
            Window::new("Pipeline JSON")
                .open(&mut open)
                .resizable(true)
                .default_size([500.0, 400.0])
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut self.json_preview)
                                .font(FontId::monospace(11.0))
                                .desired_width(f32::INFINITY)
                                .desired_rows(20),
                        );
                    });
                });
            self.show_json = open;
        }
    }
}

impl ChainythingApp {
    fn execute_pipeline(&mut self) {
        let json = match self.node_editor.to_json() {
            Ok(j) => j,
            Err(e) => {
                self.status = Status::Error(format!("Serialization error: {e}"));
                return;
            }
        };

        let registry = ProcessorRegistry::with_standard_processors();

        match PipelineBuilder::build_from_json(&json, &registry) {
            Ok(mut pipeline) => match pipeline.execute() {
                Ok(_) => self.status = Status::Success,
                Err(e) => self.status = Status::Error(match e {
                    PipelineErrors::UnknownProcessor(id) => format!("Unknown processor: {id}"),
                    PipelineErrors::ComputingError(msg) => format!("Execution error: {msg}"),
                    _ => "Unknown execution error".to_string(),
                }),
            },
            Err(e) => self.status = Status::Error(match e {
                PipelineErrors::ComputingError(msg) => format!("Build error: {msg}"),
                _ => "Unknown build error".to_string(),
            }),
        }
    }
}

fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();
    visuals.window_fill = Color32::from_rgb(20, 20, 32);
    visuals.panel_fill = Color32::from_rgb(14, 14, 22);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(28, 28, 42);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 35, 55);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 70);
    visuals.widgets.active.bg_fill = Color32::from_rgb(60, 60, 90);
    visuals.selection.bg_fill = Color32::from_rgb(60, 100, 200);
    ctx.set_visuals(visuals);
}