use anyhow::anyhow;
use anyhow::Context;
use futures::Stream;
use futures::{stream::StreamExt, TryStreamExt};
use reqwest::{Response, StatusCode};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

use super::prompts::Prompt;

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatCompletionMessage>,
    pub max_tokens: u32,
    pub stream: bool,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionParameters {
    pub r#type: String,
    pub properties: HashMap<String, ChatCompletionProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionProperty {
    pub r#type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionResponseChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponseChoice {
    delta: Option<ChatCompletionMessage>,
    message: Option<ChatCompletionMessage>,
}

#[derive(Debug)]
pub struct OpenAI {
    api_key: String,
}

impl OpenAI {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn execute<T: Prompt>(&self, prompt: T) -> anyhow::Result<T::Response> {
        //TODO: Handle timeout
        let response = call_model(&self.api_key, prompt.build_payload(), 20000).await?;
        prompt.parse_response(&response)
    }

    // Returns a stream of short strings from openai
    // If openai is trying return "foo bar baz", one chunk could be "foo b"
    pub async fn call_model_stream(
        &self,
        payload: ChatCompletionRequest,
        timeout_per_attempt_ms: u64,
        trailing_newline: bool,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = String> + Send + Sync>>> {
        let response =
            call_model_with_retries(payload, &self.api_key, timeout_per_attempt_ms, 100, 500)
                .await?;

        let body = response.bytes_stream().map_err(|err| -> std::io::Error {
            std::io::Error::new(std::io::ErrorKind::Other, err)
        });

        // Convert the stream of bytes into a stream of lines
        let async_read = StreamReader::new(body);
        let reader = tokio::io::BufReader::new(async_read);
        let lines = reader.lines();
        let lines_stream = tokio_stream::wrappers::LinesStream::new(lines);

        Ok(Box::pin(
            lines_stream
                .filter_map(move |line_result| async move {
                    match line_result {
                        Ok(line) => {
                            if line.starts_with("data: {") {
                                Some(line)
                            } else if line == "data: [DONE]" && trailing_newline {
                                Some(
                                    r#"data: {"choices":[{"delta":{"content":"\n"}}]}"#.to_string(),
                                )
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    }
                })
                .map(|line| String::from(&line[6..line.len()]))
                .map(|json| {
                    let parsed: serde_json::Result<ChatCompletionResponse> =
                        serde_json::from_str(&json);
                    parsed.expect("Error parsing inner response")
                })
                .filter_map(|parsed_response| async move {
                    let delta = &parsed_response.choices.get(0).unwrap().delta;

                    if let Some(delta) = delta {
                        if let Some(content) = &delta.content {
                            return Some(String::from(content));
                        }
                    }

                    None
                }),
        ))
    }
}

/// Calls the model, gets a String back.
/// Currently unused, but this is too likely to be used again for me to delete.
#[tracing::instrument]
async fn call_model(
    api_key: &str,
    payload: ChatCompletionRequest,
    timeout_per_attempt_ms: u64,
) -> anyhow::Result<String> {
    let response =
        call_model_with_retries(payload, api_key, timeout_per_attempt_ms, 2000, 10000).await?;

    let bytes = response.bytes().await.map_err(|err| anyhow!(err))?;
    let json = std::str::from_utf8(&bytes).map_err(|e| anyhow!(e))?;

    let parsed: ChatCompletionResponse = serde_json::from_str(json).map_err(|e| anyhow!(e))?;

    if parsed.choices.is_empty() {
        return Err(anyhow!("no choices in response"));
    }

    let choice = &parsed.choices[0];
    let message = match &choice.message {
        Some(message) => message,
        None => return Err(anyhow!("no message in choice")),
    };

    match &message.content {
        Some(content) => Ok(content.clone()),
        None => Err(anyhow!("no content in message")),
    }
}

async fn call_model_with_retries(
    payload: ChatCompletionRequest,
    api_key: &str,
    timeout_per_attempt_ms: u64,
    min_retry_delay_ms: u64,
    max_retry_delay_ms: u64,
) -> anyhow::Result<Response> {
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(
            Duration::from_millis(min_retry_delay_ms),
            Duration::from_millis(max_retry_delay_ms),
        )
        .build_with_max_retries(4);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(Duration::from_millis(timeout_per_attempt_ms))
        .json(&payload)
        .send()
        .await
        .with_context(|| "Failed to call open ai chat completion endpoint")?;

    let status = response.status();
    if status != StatusCode::OK {
        let response_body = response
            .text()
            .await
            .with_context(|| "Failed to extract text from openai error body")?;

        return Err(anyhow::anyhow!("Error calling openai: {response_body}"));
    }

    Ok(response)
}
