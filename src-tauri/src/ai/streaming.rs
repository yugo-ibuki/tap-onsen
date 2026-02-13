use futures::StreamExt;
use reqwest::Response;

use super::{AIError, StreamChunk};

/// OpenAI SSEレスポンスのチャンク構造
#[derive(serde::Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
}

#[derive(serde::Deserialize)]
struct OpenAIChoice {
    delta: OpenAIDelta,
}

#[derive(serde::Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIChoice>,
}

/// Anthropic SSEレスポンスのチャンク構造
#[derive(serde::Deserialize)]
struct AnthropicDelta {
    #[serde(default)]
    text: Option<String>,
}

#[derive(serde::Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<AnthropicDelta>,
}

/// SSE行からdataフィールドを抽出する
fn extract_sse_data(line: &str) -> Option<&str> {
    line.strip_prefix("data: ")
}

/// OpenAI SSEストリームをパースして StreamChunk に変換する
pub async fn parse_openai_stream(
    response: Response,
    sender: tokio::sync::mpsc::Sender<StreamChunk>,
) -> Result<String, AIError> {
    let mut full_text = String::new();
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AIError::StreamError(e.to_string()))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // SSEは改行区切りで送られるため、行単位で処理
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Some(data) = extract_sse_data(&line) {
                if data == "[DONE]" {
                    let _ = sender
                        .send(StreamChunk {
                            content: String::new(),
                            done: true,
                        })
                        .await;
                    return Ok(full_text);
                }

                if let Ok(parsed) = serde_json::from_str::<OpenAIStreamResponse>(data) {
                    if let Some(choice) = parsed.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            full_text.push_str(content);
                            let _ = sender
                                .send(StreamChunk {
                                    content: content.clone(),
                                    done: false,
                                })
                                .await;
                        }
                    }
                }
            }
        }
    }

    Ok(full_text)
}

/// Anthropic SSEストリームをパースして StreamChunk に変換する
pub async fn parse_anthropic_stream(
    response: Response,
    sender: tokio::sync::mpsc::Sender<StreamChunk>,
) -> Result<String, AIError> {
    let mut full_text = String::new();
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AIError::StreamError(e.to_string()))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Some(data) = extract_sse_data(&line) {
                if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data) {
                    match event.event_type.as_str() {
                        "content_block_delta" => {
                            if let Some(delta) = &event.delta {
                                if let Some(text) = &delta.text {
                                    full_text.push_str(text);
                                    let _ = sender
                                        .send(StreamChunk {
                                            content: text.clone(),
                                            done: false,
                                        })
                                        .await;
                                }
                            }
                        }
                        "message_stop" => {
                            let _ = sender
                                .send(StreamChunk {
                                    content: String::new(),
                                    done: true,
                                })
                                .await;
                            return Ok(full_text);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(full_text)
}
