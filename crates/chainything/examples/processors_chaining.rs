// Plus besoin de `extern crate chainything;` en Rust 2018+
use chainything::prelude::*;

fn main() {
    let mut pipeline = Pipeline::new();

    // 1. Reader
    let reader = ImageReaderProcessor::new("reader".to_string());
    pipeline.add_processor(
        Box::new(reader),
        vec![InputSource::static_data("./chat.jpg".to_string())],
    );

    // 2. Greyscale
    let greyscale = GreyScaleProcessor::new("greyscale".to_string());
    pipeline.add_processor(
        Box::new(greyscale),
        vec![InputSource::connection("reader", 0)],
    );

    // 3. Saver
    let saver = ImageSaveProcessor::new("saver".to_string());
    pipeline.add_processor(
        Box::new(saver),
        vec![
            InputSource::connection("greyscale", 0),
            InputSource::static_data("./output.png".to_string()),
        ],
    );

    let _ = pipeline.execute();
}
