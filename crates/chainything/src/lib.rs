pub mod llm;
pub mod pipeline_core;
pub mod processors;

pub mod prelude {
    pub use crate::pipeline_core::builder::PipelineBuilder;
    pub use crate::pipeline_core::pipeline::{InputSource, Pipeline, PipelineErrors};
    pub use crate::pipeline_core::registry::ProcessorRegistry;

    pub use crate::llm::ollama::OllamaLlm;
    pub use crate::llm::{Llm, LlmError, LlmHandle};

    pub use crate::processors::images::greyscale_processor::GreyScaleProcessor;
    pub use crate::processors::images::image_reader_processor::ImageReaderProcessor;
    pub use crate::processors::images::image_saver_processor::ImageSaveProcessor;
    pub use crate::processors::llm::llm_generate_processor::LlmGenerateProcessor;
    pub use crate::processors::llm::ollama_loader_processor::OllamaLoaderProcessor;
    pub use crate::processors::text::text_saver_processor::TextSaveProcessor;
}
