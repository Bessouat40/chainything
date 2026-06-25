use crate::nodes::{base_node::BaseNode, node_registry::NodeRegistry};
use crate::nodes::viewer::DemoViewer;
use egui::Ui;
use egui_snarl::{Snarl, ui::SnarlWidget};

pub struct DAGLayout {
    pub snarl: Snarl<Box<dyn BaseNode>>,
    viewer: DemoViewer,
}

impl Default for DAGLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DAGLayout {
    pub fn new() -> Self {
        let snarl = Snarl::new();
        let demo_viewer = DemoViewer::new();

        // snarl.insert_node(
        //     egui::pos2(100.0, 150.0),
        //     demo_viewer.node_registry.get_node().unwrap()
        // );
        // let n_path = snarl.insert_node(
        //     egui::pos2(100.0, 150.0),
        //     NodeRegistry::TextInputProcessor("C:/images/test.png".to_string())
        // );

        // snarl.insert_node(
        //     egui::pos2(400.0, 150.0),
        //     NodeRegistry::ImageReaderProcessor(String::new())
        // );

        // let n_reader = snarl.insert_node(
        //     egui::pos2(400.0, 150.0),
        //     NodeRegistry::ImageReaderProcessor(String::new())
        // );

        // snarl.insert_node(
        //     egui::pos2(700.0, 150.0),
        //     NodeRegistry::ImageDisplay(String::new())
        // );

        // let n_display = snarl.insert_node(
        //     egui::pos2(700.0, 150.0),
        //     NodeRegistry::ImageDisplay(String::new())
        // );

        // snarl.connect(
        //     OutPinId { node: n_path, output: 0 },
        //     InPinId { node: n_reader, input: 0 }
        // );

        // snarl.connect(
        //     OutPinId { node: n_reader, output: 0 },
        //     InPinId { node: n_display, input: 0 }
        // );
        Self {
            snarl,
            viewer: demo_viewer,
        }
    }

    pub fn get_snarl_and_registry(&mut self) -> (&mut Snarl<Box<dyn BaseNode>>, &NodeRegistry) {
        (&mut self.snarl, &self.viewer.node_registry)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        SnarlWidget::new().show(&mut self.snarl, &mut self.viewer, ui);
    }
}
