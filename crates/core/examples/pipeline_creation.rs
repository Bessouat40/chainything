use chainything::prelude::*;

fn main() {
    let mut pipeline = Pipeline::new();
    
    let reader = ImageReaderProcessor::new("reader".to_string());
    let input_proc1 = "./chat.jpg".to_string();
    let input_proc1 = vec![InputSource::Static(std::sync::Arc::new(input_proc1))];
    pipeline.add_processor(Box::new(reader), input_proc1);

    let greyscale_processor = GreyScaleProcessor::new("greyscale".to_string());
    let input_proc2 = vec![InputSource::Connection { node_id: "reader".to_string(), output_slot: 0 }];
    pipeline.add_processor(Box::new(greyscale_processor), input_proc2);

    let image_saver_processor = ImageSaveProcessor::new("saver".to_string());
    let input_proc3 = vec![
        InputSource::Connection { node_id: "greyscale".to_string(), output_slot: 0 },
        InputSource::Static(std::sync::Arc::new("./output.png".to_string()))
    ];
    pipeline.add_processor(Box::new(image_saver_processor), input_proc3);

    let _ = pipeline.execute();
}