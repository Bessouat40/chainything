use std::collections::HashMap;

use crate::nodes::{
    base_node::BaseNode,
    images::{
        blur_node::BlurNode, brightness_node::BrightnessNode, edge_detect_node::EdgeDetectNode,
        greyscale_node::GreyScaleNode, image_display_node::ImageDisplayNode,
        image_reader_node::ImageReaderNode, image_saver_node::ImageSaveNode,
        invert_node::InvertNode, merge_node::MergeNode, resize_node::ResizeNode,
        rotate_node::RotateNode, threshold_node::ThresholdNode,
    },
    llm::{
        llm_generate_node::LlmGenerateNode, ollama_loader_node::OllamaLoaderNode,
        vlm_generate_node::VlmGenerateNode,
    },
    model3d::{
        model_reader_node::ModelReaderNode, model_render_node::ModelRenderNode,
        model_saver_node::ModelSaveNode, model_scale_node::ModelScaleNode,
    },
    text::{
        text_display_node::TextDisplayNode, text_input_node::TextInputNode,
        text_saver_node::TextSaveNode,
    },
};

pub struct NodeRegistry {
    pub nodes: HashMap<String, Box<dyn BaseNode>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let nodes = Self::create_node_registry();
        Self { nodes }
    }

    fn create_node_registry() -> HashMap<String, Box<dyn BaseNode>> {
        [
            (
                TextInputNode::new().name().to_string(),
                Box::new(TextInputNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ImageReaderNode::new().name().to_string(),
                Box::new(ImageReaderNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ImageDisplayNode::new().name().to_string(),
                Box::new(ImageDisplayNode::new()) as Box<dyn BaseNode>,
            ),
            (
                GreyScaleNode::new().name().to_string(),
                Box::new(GreyScaleNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ImageSaveNode::new().name().to_string(),
                Box::new(ImageSaveNode::new()) as Box<dyn BaseNode>,
            ),
            (
                BlurNode::new().name().to_string(),
                Box::new(BlurNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ResizeNode::new().name().to_string(),
                Box::new(ResizeNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ThresholdNode::new().name().to_string(),
                Box::new(ThresholdNode::new()) as Box<dyn BaseNode>,
            ),
            (
                InvertNode::new().name().to_string(),
                Box::new(InvertNode::new()) as Box<dyn BaseNode>,
            ),
            (
                RotateNode::new().name().to_string(),
                Box::new(RotateNode::new()) as Box<dyn BaseNode>,
            ),
            (
                BrightnessNode::new().name().to_string(),
                Box::new(BrightnessNode::new()) as Box<dyn BaseNode>,
            ),
            (
                EdgeDetectNode::new().name().to_string(),
                Box::new(EdgeDetectNode::new()) as Box<dyn BaseNode>,
            ),
            (
                MergeNode::new().name().to_string(),
                Box::new(MergeNode::new()) as Box<dyn BaseNode>,
            ),
            (
                OllamaLoaderNode::new().name().to_string(),
                Box::new(OllamaLoaderNode::new()) as Box<dyn BaseNode>,
            ),
            (
                LlmGenerateNode::new().name().to_string(),
                Box::new(LlmGenerateNode::new()) as Box<dyn BaseNode>,
            ),
            (
                VlmGenerateNode::new().name().to_string(),
                Box::new(VlmGenerateNode::new()) as Box<dyn BaseNode>,
            ),
            (
                TextSaveNode::new().name().to_string(),
                Box::new(TextSaveNode::new()) as Box<dyn BaseNode>,
            ),
            (
                TextDisplayNode::new().name().to_string(),
                Box::new(TextDisplayNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ModelReaderNode::new().name().to_string(),
                Box::new(ModelReaderNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ModelScaleNode::new().name().to_string(),
                Box::new(ModelScaleNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ModelRenderNode::new().name().to_string(),
                Box::new(ModelRenderNode::new()) as Box<dyn BaseNode>,
            ),
            (
                ModelSaveNode::new().name().to_string(),
                Box::new(ModelSaveNode::new()) as Box<dyn BaseNode>,
            ),
        ]
        .into_iter()
        .collect()
    }

    pub fn create_node(&self, node_name: &str) -> Option<Box<dyn BaseNode>> {
        self.nodes.get(node_name).cloned()
    }
}
