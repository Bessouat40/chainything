mod processors;
use processors::image_reader_processor::ImageReaderProcessor;
use processors::greyscale_processor::GreyScaleProcessor;

mod pipeline;
use pipeline::pipeline::Pipeline;

fn main() {
    let mut pipeline = Pipeline::new();
    
    let reader = ImageReaderProcessor::new();
    let input_proc1 = "./chat.jpg".to_string();
    pipeline.add_processor("reader", Box::new(reader), input_proc1);

    let greyscale_processor = GreyScaleProcessor::new();
    let input_proc2 = (("reader", 0));
    pipeline.add_processor("greyscale", Box::new(greyscale_processor), input_proc2);

    let result = pipeline.execute();
}