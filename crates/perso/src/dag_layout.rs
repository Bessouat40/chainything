use egui::Ui;
use egui_snarl::{ui::SnarlWidget, InPinId, OutPinId, Snarl}; 
use crate::nodes::node::MyNode;
use crate::nodes::viewer::MyGraphViewer;

pub struct DAGLayout {
    snarl: Snarl<MyNode>,
    viewer: MyGraphViewer,
}

impl Default for DAGLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DAGLayout {
    pub fn new() -> Self {
        let mut snarl = Snarl::new();

        // --- INITIALISATION DU PIPELINE D'IMAGE ---
        
        // 1. Nœud d'entrée : Chemin du fichier
        let n_path = snarl.insert_node(
            egui::pos2(100.0, 150.0), 
            MyNode::String("C:/images/test.png".to_string())
        );

        // 2. Nœud processeur : Lit l'image à partir du chemin
        let n_reader = snarl.insert_node(
            egui::pos2(400.0, 150.0), 
            MyNode::ImageReader
        );

        // 3. Nœud de sortie : Affiche ou consomme l'image
        let n_display = snarl.insert_node(
            egui::pos2(700.0, 150.0), 
            MyNode::ImageDisplay
        );

        // --- CRÉATION DES LIENS ---
        
        // n_path (sortie 0: String) -> n_reader (entrée 0: String attendue)
        snarl.connect(
            OutPinId { node: n_path, output: 0 }, 
            InPinId { node: n_reader, input: 0 }
        );

        // n_reader (sortie 0: RawImage) -> n_display (entrée 0: RawImage attendue)
        snarl.connect(
            OutPinId { node: n_reader, output: 0 }, 
            InPinId { node: n_display, input: 0 }
        );

        Self {
            snarl,
            viewer: MyGraphViewer,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        SnarlWidget::new().show(&mut self.snarl, &mut self.viewer, ui);
    }
}