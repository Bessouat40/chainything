use egui::Ui;
use egui_snarl::{ui::SnarlWidget, InPinId, OutPinId, Snarl}; 
use crate::nodes::node_registry::NodeRegistry;
use crate::nodes::viewer::DemoViewer;

pub struct DAGLayout {
    snarl: Snarl<NodeRegistry>,
    viewer: DemoViewer,
}

impl Default for DAGLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DAGLayout {
    pub fn new() -> Self {
        let mut snarl = Snarl::new();
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

    pub fn show(&mut self, ui: &mut Ui) {
        let snarl: &mut Snarl<Box<dyn crate::nodes::base_node::BaseNode>> = 
            unsafe { std::mem::transmute(&mut self.snarl) };
        SnarlWidget::new().show(snarl, &mut self.viewer, ui);
    }
}