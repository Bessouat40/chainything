use crate::dag_layout::DAGLayout;
use chainything::prelude::*;
use egui::*;
use std::sync::Arc;
use std::sync::Mutex;

pub struct BottomPanel {
    is_executing: Arc<Mutex<bool>>,
}

impl Default for BottomPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl BottomPanel {
    pub fn new() -> Self {
        Self {
            is_executing: Arc::new(Mutex::new(false)),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, dag_layout: &DAGLayout) {
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .frame(
                egui::Frame::default()
                    .fill(ui.style().visuals.panel_fill)
                    .inner_margin(Margin::same(12)),
            )
            .show_inside(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let is_running = *self.is_executing.lock().unwrap();
                    let run_btn = Button::new(
                        RichText::new(if is_running { "⏸ RUNNING..." } else { "▶ RUN" })
                            .size(14.0)
                            .color(Color32::WHITE)
                            .strong(),
                    )
                    .fill(if is_running {
                        Color32::from_rgb(200, 100, 100)
                    } else {
                        Color32::from_rgb(34, 197, 94)
                    })
                    .corner_radius(8.0)
                    .min_size(Vec2::new(130.0, 38.0));

                    if ui
                        .add_enabled(!is_running, run_btn)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        let json_data = dag_layout.export_to_json();
                        let is_executing = Arc::clone(&self.is_executing);

                        std::thread::spawn(move || {
                            *is_executing.lock().unwrap() = true;
                            execute_pipeline(&json_data);
                            *is_executing.lock().unwrap() = false;
                        });
                    }
                });
            });
    }
}

fn execute_pipeline(json_data: &str) {
    println!("{:?}", json_data);
    let registry = ProcessorRegistry::with_standard_processors();

    match PipelineBuilder::build_from_json(json_data, &registry) {
        Ok(mut pipeline) => {
            println!("Pipeline built, executing...");

            match pipeline.execute() {
                Ok(_) => println!("✓ Pipeline executed successfully!"),
                Err(e) => match e {
                    PipelineErrors::UnknownProcessor(id) => {
                        eprintln!("✗ Error: Unknown processor {}", id)
                    }
                    PipelineErrors::ComputingError(msg) => eprintln!("✗ Computing error: {}", msg),
                    _ => eprintln!("✗ An error occurred!"),
                },
            }
        }
        Err(e) => match e {
            PipelineErrors::ComputingError(msg) => eprintln!("✗ Build error: {}", msg),
            _ => eprintln!("✗ Unknown error!"),
        },
    }
}
