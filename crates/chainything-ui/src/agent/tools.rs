use anyhow::Result;
use rig::{
    completion::{ToolDefinition},
    tool::Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::nodes::base_node::NodeCategory;
use crate::nodes::node_registry::NodeRegistry;

#[derive(Deserialize)]
pub struct GetNodeCategoriesArgs {}

#[derive(Debug, thiserror::Error)]
#[error("GetNodeCategories error")]
pub struct GetNodeCategoriesError;

#[derive(Deserialize, Serialize)]
pub struct GetNodeCategories;
impl Tool for GetNodeCategories {
    const NAME: &'static str = "get_node_categories";
    type Error = GetNodeCategoriesError;
    type Args = GetNodeCategoriesArgs;
    type Output = Vec<String>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_node_categories".to_string(),
            description: "Get the list of node categories".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": [],
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[tool-call] Getting node categories");
        Ok(NodeCategory::ALL.iter().map(|c| c.label().to_string()).collect())
    }
}

#[derive(Deserialize, Serialize)]
pub struct GetNodesFromCategoryArgs {
    category: String,
}

#[derive(Debug, thiserror::Error)]
#[error("GetNodesFromCategory error")]
pub struct GetNodesFromCategoryError;

#[derive(Debug, Serialize)]
pub struct NodeFromCategory {
    name: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
}


#[derive(Deserialize, Serialize)]
pub struct GetNodesFromCategory;
impl Tool for GetNodesFromCategory {
    const NAME: &'static str = "get_nodes_from_category";
    type Error = GetNodesFromCategoryError;
    type Args = GetNodesFromCategoryArgs;
    type Output = Vec<NodeFromCategory>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_nodes_from_category".to_string(),
            description: "Get the list of nodes from a specific category".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "description": "The category to get nodes from"
                    }
                },
                "required": ["category"],
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("[tool-call] Getting nodes from category");
        let node_registry = NodeRegistry::new();
        let nodes = node_registry
            .nodes
            .iter()
            .filter(|(_name, node)| {
                node.category().label() == args.category.as_str()
            })
            .map(|(name, node)| NodeFromCategory {
                name: name.clone(),
                inputs: node
                    .mapping_input()
                    .into_iter()
                    .flat_map(|mapping| {
                        let mut pins: Vec<_> = mapping.into_iter().collect();
                        pins.sort_by_key(|(index, _)| *index);
                        pins.into_iter()
                            .map(|(_, input)| input.to_string().to_string())
                    })
                    .collect(),
                outputs: node
                    .mapping_output()
                    .into_iter()
                    .flat_map(|mapping| {
                        let mut pins: Vec<_> = mapping.into_iter().collect();
                        pins.sort_by_key(|(index, _)| *index);
                        pins.into_iter()
                            .map(|(_, output)| output.to_string().to_string())
                    })
                    .collect(),
            })
            .collect();
        Ok(nodes)
    }
}