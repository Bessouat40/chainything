use chainything::prelude::*;

fn main() {
    let json_data = r#"{
        "nodes": [
            {
            "id": "reader",
            "type": "ImageReader",
            "parameters": {},
            "inputs": [
                {
                "value": "./chat.jpg"
                }
            ]
            },
            {
            "id": "greyscale",
            "type": "Greyscale",
            "parameters": {},
            "inputs": [
                {
                "source_node": "reader",
                "source_slot": 0
                }
            ]
            },
            {
            "id": "saver",
            "type": "ImageSave",
            "parameters": {},
            "inputs": [
                {
                "source_node": "greyscale",
                "source_slot": 0
                },
                {
                "value": "./output.png"
                }
            ]
            }
        ]
        }"#;

    let registry = ProcessorRegistry::with_standard_processors();

    match PipelineBuilder::build_from_json(json_data, &registry) {
        Ok(mut pipeline) => {
            println!("Pipeline built, executing...");
            
            match pipeline.execute() {
                Ok(_) => println!("Success!"),
                Err(e) => match e {
                    PipelineErrors::UnknownProcessor(id) => eprintln!("Error: Unknown processor {}", id),
                    PipelineErrors::ComputingError(msg) => eprintln!("Computing error: {}", msg),
                    _ => eprintln!("An error occurred!"),
                }
            }
        },
        Err(e) => match e {
            PipelineErrors::ComputingError(msg) => eprintln!("Build error: {}", msg),
            _ => eprintln!("Unknown error!"),
        }
    }
}