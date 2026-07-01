use egui::*;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::ollama;
use std::sync::{Arc, Mutex};

use crate::agent::prompt::PROMPT;
use crate::agent::tools::{GetNodeCategories, GetNodesFromCategory};

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

#[derive(Default)]
struct PendingResponse {
    response: Option<String>,
    error: Option<String>,
}

#[derive(Default)]
pub struct LlmModal {
    pub is_open: bool,
    pub input_text: String,
    pub model_name: String,
    pub chat_history: Vec<ChatMessage>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pending_response: Arc<Mutex<PendingResponse>>,
}

impl LlmModal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            input_text: String::new(),
            model_name: String::new(),
            chat_history: Vec::new(),
            is_loading: false,
            error_message: None,
            pending_response: Arc::new(Mutex::new(PendingResponse {
                response: None,
                error: None,
            })),
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn add_user_message(&mut self, content: String) {
        self.chat_history.push(ChatMessage {
            role: "user".to_string(),
            content,
        });
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.chat_history.push(ChatMessage {
            role: "assistant".to_string(),
            content,
        });
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        if self.is_loading {
            let (response, error) = {
                let mut pending = self.pending_response.lock().unwrap();
                (pending.response.take(), pending.error.take())
            };
            if let Some(response_text) = response {
                self.add_assistant_message(response_text);
                self.is_loading = false;
                self.error_message = None;
            } else if let Some(error) = error {
                self.error_message = Some(error);
                self.is_loading = false;
            }
        }

        let mut json_to_apply = None;
        let mut is_open = self.is_open;
        if is_open {
            Window::new("Generate Graph with AI")
                .open(&mut is_open)
                .resizable(true)
                .default_size([520.0, 480.0])
                .show(ctx, |ui| {
                    let history_height = (ui.available_height() - 130.0).max(120.0);
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(history_height)
                        .show(ui, |ui| {
                            for message in &mut self.chat_history {
                                ui.horizontal_wrapped(|ui| {
                                    let label_text = if message.role == "user" {
                                        RichText::new("You:")
                                            .strong()
                                            .color(Color32::from_rgb(100, 150, 255))
                                    } else {
                                        RichText::new("AI:")
                                            .strong()
                                            .color(Color32::from_rgb(100, 200, 100))
                                    };
                                    ui.label(label_text);
                                });

                                ui.add(
                                    TextEdit::multiline(&mut message.content)
                                        .desired_rows(3)
                                        .interactive(false)
                                        .font(TextStyle::Monospace.resolve(ui.style())),
                                );
                                if message.role == "assistant"
                                    && let Some(extracted_json) =
                                        extract_json_from_markdown(&message.content)
                                {
                                    ui.add_space(4.0);
                                    if ui.button("Apply").clicked() {
                                        json_to_apply = Some(extracted_json);
                                    }
                                }
                                ui.separator();
                            }
                        });

                    if let Some(error) = &self.error_message {
                        ui.colored_label(Color32::from_rgb(255, 100, 100), error);
                        if ui.button("Clear error").clicked() {
                            self.clear_error();
                        }
                        ui.separator();
                    }

                    ui.horizontal(|ui| {
                        ui.label("Model:");
                        ui.text_edit_singleline(&mut self.model_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Prompt:");
                        ui.text_edit_singleline(&mut self.input_text);

                        if ui
                            .add_enabled(
                                !self.is_loading && !self.model_name.is_empty(),
                                Button::new("Send"),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                            && !self.input_text.is_empty()
                        {
                            let user_input = self.input_text.clone();
                            let model_name = self.model_name.clone();
                            let message_history_prompt = self
                                .chat_history
                                .iter()
                                .map(|msg| format!("{}: {}", msg.role, msg.content))
                                .collect::<Vec<String>>()
                                .join("\n");
                            self.add_user_message(user_input.clone());
                            self.input_text.clear();
                            self.is_loading = true;

                            let pending = Arc::clone(&self.pending_response);
                            std::thread::spawn(move || {
                                let result: Result<String, String> = (|| {
                                    let rt = tokio::runtime::Runtime::new()
                                        .map_err(|e| e.to_string())?;

                                    rt.block_on(async {
                                        let client = ollama::Client::from_env()
                                            .map_err(|e| e.to_string())?;

                                        let agent = client
                                            .agent(model_name.as_str())
                                            .preamble(PROMPT)
                                            .max_tokens(2048)
                                            .default_max_turns(10)
                                            .tool(GetNodeCategories)
                                            .tool(GetNodesFromCategory)
                                            .build();

                                        let enrich_user_input = format!(
                                            "Here is the history : {}, and here is the new user input : {}",
                                            message_history_prompt, user_input
                                        );

                                        agent
                                            .prompt(enrich_user_input.as_str())
                                            .await
                                            .map_err(|e| e.to_string())
                                    })
                                })(
                                );

                                let mut pending_lock = pending.lock().unwrap();
                                match result {
                                    Ok(response) => pending_lock.response = Some(response),
                                    Err(err) => pending_lock.error = Some(err),
                                }
                            });
                        }

                        if self.is_loading {
                            ui.add(Spinner::new());
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui
                            .button("Clear History")
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.chat_history.clear();
                            self.input_text.clear();
                        }

                        if ui
                            .button("Close")
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.close();
                        }
                    });
                });
        }

        self.is_open = is_open;
        json_to_apply
    }
}

fn extract_json_from_markdown(content: &str) -> Option<String> {
    let start_tag = "```json";
    let end_tag = "```";

    if let Some(start_idx) = content.find(start_tag) {
        let content_after_start = &content[start_idx + start_tag.len()..];
        if let Some(end_idx) = content_after_start.find(end_tag) {
            return Some(content_after_start[..end_idx].trim().to_string());
        }
    }
    None
}
