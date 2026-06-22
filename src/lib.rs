pub mod processors;
pub mod pipeline;

pub mod prelude {
    pub use crate::pipeline::pipeline::{Pipeline, InputSource, PipelineErrors};
    pub use crate::pipeline::builder::PipelineBuilder;
    pub use crate::pipeline::registry::ProcessorRegistry;
    
    pub use crate::processors::image_reader_processor::ImageReaderProcessor;
    pub use crate::processors::greyscale_processor::GreyScaleProcessor;
    pub use crate::processors::image_saver_processor::ImageSaveProcessor;
}