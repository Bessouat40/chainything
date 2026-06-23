use egui::Ui;
use egui_snarl::{
    InPin, OutPin, Snarl,
    ui::{PinInfo, SnarlViewer},
};

#[derive(Clone)]
pub enum MyNode {
    String(String),
    ImageReader,
    ImageDisplay,
}

pub struct MyGraphViewer;

impl SnarlViewer<MyNode> for MyGraphViewer {
    fn title(&mut self, node: &MyNode) -> String {
        match node {
            MyNode::String(_) => "Chemin de l'image".to_owned(),
            MyNode::ImageReader => "Processeur : Image Reader".to_owned(),
            MyNode::ImageDisplay => "Affichage Image".to_owned(),
        }
    }

    fn inputs(&mut self, node: &MyNode) -> usize {
        match node {
            MyNode::String(_) => 0,
            MyNode::ImageReader => 1,
            MyNode::ImageDisplay => 1,
        }
    }

    fn outputs(&mut self, node: &MyNode) -> usize {
        match node {
            MyNode::String(_) => 1,
            MyNode::ImageReader => 1,
            MyNode::ImageDisplay => 0,
        }
    }

    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, _snarl: &mut Snarl<MyNode>) -> PinInfo {
        match _snarl[pin.id.node] {
            MyNode::ImageReader => {
                ui.label("Path (String)");
                PinInfo::circle().with_fill(egui::Color32::GREEN)
            }
            MyNode::ImageDisplay => {
                ui.label("RawImage");
                PinInfo::circle().with_fill(egui::Color32::LIGHT_BLUE)
            }
            _ => PinInfo::circle().with_fill(egui::Color32::GRAY),
        }
    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<MyNode>) -> PinInfo {
        match snarl[pin.id.node] {
            MyNode::String(ref mut value) => {
                ui.add(egui::TextEdit::singleline(value).desired_width(100.0));
                PinInfo::circle().with_fill(egui::Color32::GREEN)
            }
            MyNode::ImageReader => {
                ui.label("RawImage Out");
                PinInfo::circle().with_fill(egui::Color32::LIGHT_BLUE)
            }
            _ => PinInfo::circle().with_fill(egui::Color32::GRAY),
        }
    }
}