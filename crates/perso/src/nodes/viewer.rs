#![allow(clippy::use_self)]

use crate::nodes::node::MyNode;

use egui::{Color32, Ui};
use egui_snarl::{
    InPin, InPinId, NodeId, OutPin, OutPinId, Snarl,
    ui::{
        AnyPins, PinInfo, SnarlViewer,
        WireStyle,
    },
};

const STRING_COLOR: Color32 = Color32::from_rgb(0x00, 0xb0, 0x00);

pub struct DemoViewer;

impl SnarlViewer<MyNode> for DemoViewer {
    
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<MyNode>) {
        type PinCompat = usize;
        const PIN_STR: PinCompat = 2;
        const PIN_IMG: PinCompat = 4;
        const fn pin_out_compat(node: &MyNode) -> PinCompat {
            match node {
                MyNode::TextInputProcessor(_) => PIN_STR,
                MyNode::ImageReaderProcessor(_) => PIN_IMG,
                MyNode::ImageDisplay(_) => 0,
            }
        }

        const fn pin_in_compat(node: &MyNode, _input_index: usize) -> PinCompat {
            match node {
                MyNode::TextInputProcessor(_) => PIN_STR,
                MyNode::ImageReaderProcessor(_) => PIN_STR,
                MyNode::ImageDisplay(_) => PIN_STR,
            }
        }

        let from_node = &snarl[from.id.node];
        let to_node = &snarl[to.id.node];

        let out_compat = pin_out_compat(from_node);
        let in_compat = pin_in_compat(to_node, to.id.input);

        if (out_compat & in_compat) != 0 {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }
            snarl.connect(from.id, to.id);
        }
    }

    fn title(&mut self, node: &MyNode) -> String {
        match node {
            MyNode::TextInputProcessor(_) => "TextInputProcessor".to_owned(),
            MyNode::ImageReaderProcessor(_) => "ImageReaderProcessor".to_owned(),
            MyNode::ImageDisplay(_) => "ImageDisplay".to_owned(),
        }
    }

    fn inputs(&mut self, node: &MyNode) -> usize {
        match node {
            MyNode::TextInputProcessor(_) => 0,
            MyNode::ImageReaderProcessor(_) => 1,
            MyNode::ImageDisplay(_) => 1,
        }
    }

    fn outputs(&mut self, node: &MyNode) -> usize {
        match node {
            MyNode::TextInputProcessor(_) | MyNode::ImageReaderProcessor(_) => 1,
            MyNode::ImageDisplay(_) => 0,
        }
    }

    #[allow(refining_impl_trait)]
    fn show_input(&mut self, pin: &InPin, ui: &mut Ui, snarl: &mut Snarl<MyNode>) -> PinInfo {
        match snarl[pin.id.node] {
            MyNode::TextInputProcessor(_) => {
                unreachable!("TextInputProcessor node has no inputs")
            }
            MyNode::ImageReaderProcessor(_) => {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label("Input String");
                    
                    ui.add_space(5.0);
                });

                PinInfo::circle()
                    .with_fill(STRING_COLOR)
                    .with_wire_style(WireStyle::AxisAligned {
                        corner_radius: 10.0,
                    })
            }
            MyNode::ImageDisplay(_) => {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label("Input String");
                    
                    ui.add_space(5.0);
                });

                PinInfo::circle()
                    .with_fill(STRING_COLOR)
                    .with_wire_style(WireStyle::AxisAligned {
                        corner_radius: 10.0,
                    })
            }
        }
    }

    #[allow(refining_impl_trait)]
    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, snarl: &mut Snarl<MyNode>) -> PinInfo {
        match snarl[pin.id.node] {
            MyNode::ImageReaderProcessor(ref mut _value) => {
                assert_eq!(pin.id.output, 0, "ImageReaderProcessor node has only one output");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Output Raw Image");
                });

                PinInfo::circle()
                    .with_fill(STRING_COLOR)
                    .with_wire_style(WireStyle::AxisAligned {
                        corner_radius: 10.0,
                    })
            }
            MyNode::TextInputProcessor(ref mut value) => {
                assert_eq!(pin.id.output, 0, "TextInputProcessor node has only one output");
                
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    ui.label("Output String:");
                    
                    let edit = egui::TextEdit::singleline(value)
                        .text_color(STRING_COLOR)
                        .desired_width(120.0) 
                        .font(egui::TextStyle::Monospace);
                    
                    ui.add(edit);
                });

                PinInfo::circle()
                    .with_fill(STRING_COLOR)
                    .with_wire_style(WireStyle::AxisAligned {
                        corner_radius: 10.0,
                    })
            }
            MyNode::ImageDisplay(_) => {
                unreachable!("ImageDisplay node has no outputs")
            }
        }
    }

    fn show_body(
        &mut self,
        node: NodeId,
        inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<MyNode>,
    ) {
        if let MyNode::ImageDisplay(_) = snarl[node] {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.set_width(200.0);
                let input = &inputs[0];
                let url_to_display = match input.remotes.as_slice() {
                    [remote] => {
                        Some(snarl[remote.node].string_in().clone())
                    }
                    _ => None,
                };

                if let Some(uri) = url_to_display {
                    ui.add(
                        egui::Image::new(&uri)
                            .show_loading_spinner(true)
                    );
                } else {
                    ui.label("No image to display");
                }
            });
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<MyNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<MyNode>) {
        ui.label("Add node");
        if ui.button("Text Input").clicked() {
            snarl.insert_node(pos, MyNode::TextInputProcessor(String::new()));
            ui.close();
        }
        if ui.button("Image Reader").clicked() {
            snarl.insert_node(pos, MyNode::ImageReaderProcessor(String::new()));
            ui.close();
        }
    }

    fn has_dropped_wire_menu(&mut self, _src_pins: AnyPins, _snarl: &mut Snarl<MyNode>) -> bool {
        true
    }

    #[inline]
    fn has_body(&mut self, node: &MyNode) -> bool {
        match node {
            MyNode::ImageDisplay(_) => true,
            _ => false,
        }
    }

    fn show_dropped_wire_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        src_pins: AnyPins,
        snarl: &mut Snarl<MyNode>,
    ) {
        type PinCompat = usize;
        const PIN_STR: PinCompat = 2;
        const PIN_IMG: PinCompat = 4;

        const fn pin_out_compat(node: &MyNode) -> PinCompat {
            match node {
                MyNode::TextInputProcessor(_) => PIN_STR,
                MyNode::ImageReaderProcessor(_) => PIN_IMG,
                MyNode::ImageDisplay(_) => 0,
            }
        }

        const fn pin_in_compat(node: &MyNode, _pin: usize) -> PinCompat {
            match node {
                MyNode::TextInputProcessor(_) => PIN_STR,
                MyNode::ImageReaderProcessor(_) => PIN_STR,
                MyNode::ImageDisplay(_) => PIN_STR,
            }
        }

        ui.label("Add node");

        match src_pins {
            AnyPins::Out(src_pins) => {
                if src_pins.len() != 1 {
                    ui.label("Multiple output pins are not supported in this demo");
                    return;
                }

                let src_pin = src_pins[0];
                let src_out_ty = pin_out_compat(snarl.get_node(src_pin.node).unwrap());
                let dst_in_candidates: &[(&str, fn() -> MyNode, PinCompat)] = &[
                    ("Image Reader", || MyNode::ImageReaderProcessor(String::new()), PIN_STR),
                ];

                for (name, ctor, in_ty) in dst_in_candidates {
                    if src_out_ty & in_ty != 0 && ui.button(*name).clicked() {
                        let new_node = snarl.insert_node(pos, ctor());
                        let dst_pin = InPinId {
                            node: new_node,
                            input: 0,
                        };

                        snarl.connect(src_pin, dst_pin);
                        ui.close();
                    }
                }
            }
            AnyPins::In(pins) => {
                let all_src_types = pins.iter().fold(0, |acc, pin| {
                    acc | pin_in_compat(snarl.get_node(pin.node).unwrap(), pin.input)
                });

                let dst_out_candidates = &[
                    ("Text Input", Box::new(|| MyNode::TextInputProcessor(String::new())) as Box<dyn Fn() -> MyNode>, PIN_STR),
                    ("Image Reader", Box::new(|| MyNode::ImageReaderProcessor(String::new())) as Box<dyn Fn() -> MyNode>, PIN_STR),
                ];

                for (name, ctor, out_ty) in dst_out_candidates {
                    if all_src_types & out_ty != 0 && ui.button(*name).clicked() {
                        let new_node = ctor();
                        let dst_ty = pin_out_compat(&new_node);

                        let new_node = snarl.insert_node(pos, new_node);
                        let dst_pin = OutPinId {
                            node: new_node,
                            output: 0,
                        };

                        for src_pin in pins {
                            let src_ty =
                                pin_in_compat(snarl.get_node(src_pin.node).unwrap(), src_pin.input);
                            if src_ty & dst_ty != 0 {
                                snarl.drop_inputs(*src_pin);
                                snarl.connect(dst_pin, *src_pin);
                                ui.close();
                            }
                        }
                    }
                }
            }
        }
    }

    fn has_node_menu(&mut self, _node: &MyNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<MyNode>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }

    fn has_on_hover_popup(&mut self, _: &MyNode) -> bool {
        true
    }

    fn header_frame(
        &mut self,
        frame: egui::Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<MyNode>,
    ) -> egui::Frame {
        match snarl[node] {
            MyNode::TextInputProcessor(_) => frame.fill(egui::Color32::from_rgb(70, 70, 80)),
            MyNode::ImageReaderProcessor(_) => frame.fill(egui::Color32::from_rgb(70, 40, 40)),
            MyNode::ImageDisplay(_) => frame.fill(egui::Color32::from_rgb(40, 70, 40)),
        }
    }
}