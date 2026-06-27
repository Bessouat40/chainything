use crate::bottom_panel::BottomPanel;
use crate::dag_layout::DAGLayout;
use crate::left_panel::LeftPanel;

#[derive(Default)]
pub struct ChainythingApp {
    left_panel: LeftPanel,
    bottom_panel: BottomPanel,
    dag_layout: DAGLayout,
}

impl ChainythingApp {
    pub fn new(_ctx: &eframe::CreationContext<'_>) -> Self {
        Self {
            bottom_panel: BottomPanel::new(),
            dag_layout: DAGLayout::new(),
            left_panel: LeftPanel::new(),
        }
    }
}

impl eframe::App for ChainythingApp {
    fn ui(&mut self, ctx: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            // Route any finished run's outputs into the display nodes.
            self.dag_layout.poll_results();

            let (snarl, node_registry) = self.dag_layout.get_snarl_and_registry();
            self.left_panel.show(ui, snarl, node_registry);
            self.bottom_panel.show(ui, &mut self.dag_layout);
            self.dag_layout.show(ui);

            // Keep repainting while a background run is in progress so results
            // appear as soon as it completes, even without user interaction.
            if self.dag_layout.is_running() {
                ui.ctx().request_repaint();
            }
        });
    }
}
