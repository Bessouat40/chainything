// use rig::client::{CompletionClient, ProviderClient};
// use rig::completion::Prompt;
// use rig::providers::ollama;

// use crate::agent::prompt::PROMPT;
// use crate::agent::tools::{GetNodeCategories, GetNodesFromCategory};

// #[tokio::main]
// async fn main() -> Result<(), anyhow::Error> {
//     let client = ollama::Client::from_env()?;

//     let agent = client
//         .agent("gemma4:e2b")
//         .preamble(PROMPT)
//         .tool(GetNodeCategories)
//         .tool(GetNodesFromCategory)
//         .build();

//     let response = agent.prompt("Create for me a pipeline to load an image, apply a threshold and save the result.").await?;

//     println!("{response}");

//     Ok(())
// }