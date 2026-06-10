mod processors;
use processors::image_reader_processor::ImageReaderProcessor;
use processors::greyscale_processor::GreyScaleProcessor;

mod pipeline;
use pipeline::pipeline::Pipeline;

fn main() {
    let reader = ImageReaderProcessor::new();
    let greyscale_processor = GreyScaleProcessor::new();

    let mut pipeline = Pipeline::new();
    pipeline.add_processor("reader", Box::new(reader), None);
    pipeline.add_processor("greyscale", Box::new(greyscale_processor), Some("reader"));

    let result = pipeline.execute();
}