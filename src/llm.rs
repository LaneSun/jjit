use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAYS: [u64; 3] = [5, 30, 300]; // seconds: 5s, 30s, 5min

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    thinking_type: String,
}

// Streaming response types
#[derive(Debug, Deserialize)]
struct StreamChatResponse {
    choices: Vec<StreamChatChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChatChoice {
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

pub struct LlmClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl LlmClient {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
            model,
        }
    }

    pub async fn chat(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        verbose: bool,
        show_prompt: bool,
        show_thinking: bool,
        debug: bool,
    ) -> Result<String> {
        if show_prompt {
            eprintln!("\n=== System Prompt ===");
            eprintln!("{}", system_prompt);
            eprintln!("\n=== User Prompt ===");
            eprintln!("{}", user_prompt);
            eprintln!();
        }

        if verbose {
            eprintln!("Thinking...");
        }

        if debug {
            eprintln!("[DEBUG] Model: {}, Base URL: {}", self.model, self.base_url);
        }

        let mut last_error = None;

        for attempt in 0..=MAX_RETRIES {
            match self.chat_stream(system_prompt, user_prompt, show_thinking, debug).await {
                Ok(response) => {
                    if verbose {
                        eprintln!("LLM response received");
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let is_retryable = Self::is_retryable_error(&e);
                    last_error = Some(e);

                    if attempt < MAX_RETRIES && is_retryable {
                        let delay = RETRY_DELAYS[attempt as usize];
                        eprintln!(
                            "LLM request failed (attempt {}/{}), retrying in {}s...",
                            attempt + 1,
                            MAX_RETRIES + 1,
                            delay
                        );
                        sleep(Duration::from_secs(delay)).await;
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("LLM request failed after all retries")))
    }

    async fn chat_stream(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        show_thinking: bool,
        debug: bool,
    ) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            stream: true,
            // Thinking mode doesn't support temperature
            temperature: if show_thinking { None } else { Some(0.7) },
            reasoning_effort: if show_thinking { Some("high".to_string()) } else { None },
            thinking: if show_thinking {
                Some(ThinkingConfig {
                    thinking_type: "enabled".to_string(),
                })
            } else {
                None
            },
        };

        if debug {
            eprintln!("[DEBUG] Sending streaming request to {}/chat/completions", self.base_url);
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .with_context(|| "Failed to send request to DeepSeek API")?;

        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "DeepSeek API returned error ({}): {}",
                status,
                text
            ));
        }

        // Stream the response
        let no_color = std::env::var("JJIT_NO_COLOR").is_ok() 
            || std::env::var("NO_COLOR").is_ok();
        
        let mut content = String::new();
        let mut thinking_started = false;
        let mut thinking_ended = false;
        let mut line_buffer = String::new();

        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.with_context(|| "Failed to read response stream")?;
            let text = String::from_utf8_lossy(&chunk);
            
            for ch in text.chars() {
                if ch == '\n' {
                    // Process complete line
                    let line = line_buffer.trim().to_string();
                    line_buffer.clear();
                    
                    if line.is_empty() || !line.starts_with("data: ") {
                        continue;
                    }
                    
                    let data = &line[6..]; // Skip "data: "
                    
                    if data == "[DONE]" {
                        if thinking_started && !thinking_ended {
                            if !no_color {
                                eprint!("\x1b[0m"); // Reset color
                            }
                            eprintln!();
                            let _ = std::io::stderr().flush();
                        }
                        if debug {
                            eprintln!("[DEBUG] Response content length: {}", content.len());
                        }
                        return Ok(content);
                    }
                    
                    match serde_json::from_str::<StreamChatResponse>(data) {
                        Ok(stream_resp) => {
                            if let Some(choice) = stream_resp.choices.first() {
                                // Handle reasoning content (thinking)
                                if show_thinking {
                                    if let Some(ref reasoning) = choice.delta.reasoning_content {
                                        if !reasoning.is_empty() {
                                            if !thinking_started {
                                                thinking_started = true;
                                                if !no_color {
                                                    eprint!("\x1b[90m"); // Gray color
                                                }
                                                let _ = std::io::stderr().flush();
                                            }
                                            eprint!("{}", reasoning);
                                            let _ = std::io::stderr().flush();
                                        }
                                    }
                                }
                                
                                // Handle content
                                if let Some(ref text) = choice.delta.content {
                                    if !text.is_empty() {
                                        // If we were showing thinking, end it
                                        if thinking_started && !thinking_ended {
                                            thinking_ended = true;
                                            if !no_color {
                                                eprint!("\x1b[0m"); // Reset color
                                            }
                                            eprintln!();
                                            let _ = std::io::stderr().flush();
                                        }
                                        content.push_str(text);
                                    }
                                }
                                
                                // Check if this is the final chunk
                                if choice.finish_reason.is_some() {
                                    if thinking_started && !thinking_ended {
                                        if !no_color {
                                            eprint!("\x1b[0m"); // Reset color
                                        }
                                        eprintln!();
                                        let _ = std::io::stderr().flush();
                                    }
                                    if debug {
                                        eprintln!("[DEBUG] Response content length: {}", content.len());
                                    }
                                    return Ok(content);
                                }
                            }
                        }
                        Err(e) => {
                            if debug {
                                eprintln!("[DEBUG] Failed to parse stream chunk: {}", e);
                            }
                        }
                    }
                } else {
                    line_buffer.push(ch);
                }
            }
        }

        if thinking_started && !thinking_ended {
            if !no_color {
                eprint!("\x1b[0m"); // Reset color
            }
            eprintln!();
        }

        if debug {
            eprintln!("[DEBUG] Response content length: {}", content.len());
        }

        Ok(content)
    }

    fn is_retryable_error(error: &anyhow::Error) -> bool {
        let error_string = error.to_string().to_lowercase();
        error_string.contains("timeout")
            || error_string.contains("429")
            || error_string.contains("rate limit")
            || error_string.contains("500")
            || error_string.contains("502")
            || error_string.contains("503")
            || error_string.contains("504")
            || error_string.contains("connection")
    }
}
