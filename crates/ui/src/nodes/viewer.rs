#![allow(clippy::use_self)]

use crate::nodes::node::MyNode;
use crate::nodes::node_registry::NodeRegistry;
use crate::nodes::base_node::BaseNode;

use std::mem::discriminant;

use egui::{Color32, Ui};
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{
        AnyPins, PinInfo, SnarlViewer,
        WireStyle,
    },
};

const STRING_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);

pub struct DemoViewer {
    pub node_registry: NodeRegistry,
}

impl DemoViewer {
    pub fn new() -> Self {
        Self {
            node_registry: NodeRegistry::new(),
        }
    }
}

impl SnarlViewer<Box<dyn BaseNode>> for DemoViewer {
    
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<Box<dyn BaseNode>>) {
    let out_pin_idx = from.id.output;
    let in_pin_idx = to.id.input;

    let from_node = &snarl[from.id.node];
    let to_node = &snarl[to.id.node];

    if let (Some(out_map), Some(in_map)) = (from_node.mapping_output(), to_node.mapping_input()) {
        
        if let (Some(out_type), Some(in_type)) = (out_map.get(&out_pin_idx), in_map.get(&in_pin_idx)) {
            
            if discriminant(out_type) == discriminant(in_type) {
                snarl.connect(from.id, to.id);
            }
        }
    }
}

    fn title(&mut self, node: &Box<dyn BaseNode>) -> String {
        node.name().to_string()
    }

    fn inputs(&mut self, node: &Box<dyn BaseNode>) -> usize {
        node.inputs_count()
    }

    fn outputs(&mut self, node: &Box<dyn BaseNode>) -> usize {
        node.outputs_count()
    }

    #[allow(refining_impl_trait)]
    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<Box<dyn BaseNode>>) -> PinInfo {
        snarl[pin.id.node].show_input(pin, ui)
    }

    #[allow(refining_impl_trait)]
    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<Box<dyn BaseNode>>) -> PinInfo {
        snarl[pin.id.node].show_output(pin, ui)
    }

    // fn show_body(
    //     &mut self,
    //     node: NodeId,
    //     inputs: &[InPin],
    //     _outputs: &[OutPin],
    //     ui: &mut Ui,
    //     snarl: &mut Snarl<Box<dyn BaseNode>>,
    // ) {
    //     if let MyNode::ImageDisplay(_) = snarl[node] {
    //         ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    //             ui.set_width(200.0);
    //             let input = &inputs[0];
    //             let url_to_display = match input.remotes.as_slice() {
    //                 [remote] => {
    //                     Some(snarl[remote.node].string_in().clone())
    //                 }
    //                 _ => None,
    //             };

    //             if let Some(uri) = url_to_display {
    //                 ui.add(
    //                     egui::Image::new(&uri)
    //                         .show_loading_spinner(true)
    //                 );
    //             } else {
    //                 ui.label("No image to display");
    //             }
    //         });
    //     }
    // }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<Box<dyn BaseNode>>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<Box<dyn BaseNode>>) {
    for name in self.node_registry.get_available_nodes() {
        if ui.button(name).clicked() {
            let node = self.node_registry.create_node(name);
            if let Some(created_node) = node {
                snarl.insert_node(pos, created_node);
            } 
        }
    }
}

    fn has_dropped_wire_menu(&mut self, _src_pins: AnyPins, _snarl: &mut Snarl<Box<dyn BaseNode>>) -> bool {
        true
    }

    // #[inline]
    // fn has_body(&mut self, node: &Box<dyn BaseNode>) -> bool {
    //     node.has_body()
    // }

    // fn show_dropped_wire_menu(
    //     &mut self,
    //     pos: egui::Pos2,
    //     ui: &mut Ui,
    //     src_pins: AnyPins,
    //     snarl: &mut Snarl<Box<dyn BaseNode>>,
    // ) {

    //     ui.label("Add node");

    //     match src_pins {
    //         AnyPins::Out(src_pins) => {
    //             if src_pins.len() != 1 {
    //                 ui.label("Multiple output pins are not supported in this demo");
    //                 return;
    //             }

    //             let src_pin = src_pins[0];
    //             let src_out_ty = pin_out_compat(self.node_registry.get_node(src_pin.node).unwrap());
    //             let dst_in_candidates: &[(&str, fn() -> MyNode, PinCompat)] = &[
    //                 ("Image Reader", || MyNode::ImageReaderProcessor(String::new()), PIN_STR),
    //             ];

    //             for (name, ctor, in_ty) in dst_in_candidates {
    //                 if src_out_ty & in_ty != 0 && ui.button(*name).clicked() {
    //                     let new_node = snarl.insert_node(pos, ctor());
    //                     let dst_pin = InPinId {
    //                         node: new_node,
    //                         input: 0,
    //                     };

    //                     snarl.connect(src_pin, dst_pin);
    //                     ui.close();
    //                 }
    //             }
    //         }
    //         AnyPins::In(pins) => {
    //             let all_src_types = pins.iter().fold(0, |acc, pin| {
    //                 acc | pin_in_compat(snarl.get_node(pin.node).unwrap(), pin.input)
    //             });

    //             let dst_out_candidates = &[
    //                 ("Text Input", Box::new(|| MyNode::TextInputProcessor(String::new())) as Box<dyn Fn() -> MyNode>, PIN_STR),
    //                 ("Image Reader", Box::new(|| MyNode::ImageReaderProcessor(String::new())) as Box<dyn Fn() -> MyNode>, PIN_STR),
    //             ];

    //             for (name, ctor, out_ty) in dst_out_candidates {
    //                 if all_src_types & out_ty != 0 && ui.button(*name).clicked() {
    //                     let new_node = ctor();
    //                     let dst_ty = pin_out_compat(&new_node);

    //                     let new_node = snarl.insert_node(pos, new_node);
    //                     let dst_pin = OutPinId {
    //                         node: new_node,
    //                         output: 0,
    //                     };

    //                     for src_pin in pins {
    //                         let src_ty =
    //                             pin_in_compat(snarl.get_node(src_pin.node).unwrap(), src_pin.input);
    //                         if src_ty & dst_ty != 0 {
    //                             snarl.drop_inputs(*src_pin);
    //                             snarl.connect(dst_pin, *src_pin);
    //                             ui.close();
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    fn has_node_menu(&mut self, _node: &Box<dyn BaseNode>) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<Box<dyn BaseNode>>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }

    fn has_on_hover_popup(&mut self, _: &Box<dyn BaseNode>) -> bool {
        true
    }

    fn header_frame(
        &mut self,
        frame: egui::Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<Box<dyn BaseNode>>,
    ) -> egui::Frame {
        let get_node = &snarl[node];
        get_node.header_frame(frame)
    }
}