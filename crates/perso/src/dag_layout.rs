use egui::Ui;
use egui_snarl::{ui::SnarlWidget, InPinId, OutPinId, Snarl}; 
use crate::nodes::node::MyNode;
use crate::nodes::viewer::DemoViewer;

pub struct DAGLayout {
    snarl: Snarl<MyNode>,
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

        snarl.insert_node(
            egui::pos2(100.0, 150.0), 
            MyNode::TextInputProcessor("C:/images/test.png".to_string())
        );
        // let n_path = snarl.insert_node(
        //     egui::pos2(100.0, 150.0), 
        //     MyNode::TextInputProcessor("C:/images/test.png".to_string())
        // );

        snarl.insert_node(
            egui::pos2(400.0, 150.0), 
            MyNode::ImageReaderProcessor(String::new())
        );

        // let n_reader = snarl.insert_node(
        //     egui::pos2(400.0, 150.0), 
        //     MyNode::ImageReaderProcessor(String::new())
        // );

        snarl.insert_node(
            egui::pos2(700.0, 150.0), 
            MyNode::ImageDisplay(String::new())
        );

        // let n_display = snarl.insert_node(
        //     egui::pos2(700.0, 150.0), 
        //     MyNode::ImageDisplay(String::new())
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
            viewer: DemoViewer,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        SnarlWidget::new().show(&mut self.snarl, &mut self.viewer, ui);
    }
}