pub mod llm;
pub mod pipeline_core;
pub mod processors;

pub mod prelude {
    pub use crate::pipeline_core::builder::PipelineBuilder;
    pub use crate::pipeline_core::pipeline::{InputSource, Pipeline, PipelineErrors};
    pub use crate::pipeline_core::registry::ProcessorRegistry;

    pub use crate::llm::ollama::OllamaLlm;
    pub use crate::llm::{Llm, LlmError, LlmHandle};

    pub use crate::processors::greyscale_processor::GreyScaleProcessor;
    pub use crate::processors::image_reader_processor::ImageReaderProcessor;
    pub use crate::processors::image_saver_processor::ImageSaveProcessor;
    pub use crate::processors::llm_generate_processor::LlmGenerateProcessor;
    pub use crate::processors::ollama_loader_processor::OllamaLoaderProcessor;
    pub use crate::processors::text_saver_processor::TextSaveProcessor;
}
