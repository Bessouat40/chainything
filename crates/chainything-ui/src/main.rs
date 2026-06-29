use eframe::egui;
mod app;
mod bottom_panel;
mod dag_layout;
mod graph_io;
mod left_panel;
mod nodes;
mod payload_parser;
mod agent;
mod llm_modal;

use app::ChainythingApp;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Chainything"),
        ..Default::default()
    };
    eframe::run_native(
        "Chainything",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(ChainythingApp::new(cc)))
        }),
    )
}

// use rig::client::{CompletionClient, ProviderClient};
// use rig::completion::Prompt;
// use rig::providers::ollama;

// use crate::agent::prompt::PROMPT;
// use crate::agent::tools::{GetNodeCategories, GetNodesFromCategory};

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let client = ollama::Client::from_env()?;

//     let agent = client
//         .agent("qwen3.5:9b")
//         .preamble(PROMPT)
//         .max_tokens(2048)
//         .default_max_turns(10)
//         .tool(GetNodeCategories)
//         .tool(GetNodesFromCategory)
//         .build();

//     let response = agent.prompt("Create for me a pipeline to load an image, apply a threshold and save the result.").await?;

//     println!("{response}");

//     Ok(())
// }
