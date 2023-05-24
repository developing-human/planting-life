use futures::{stream::StreamExt, TryStreamExt};

use futures::Stream;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
    max_tokens: u32,
    stream: bool,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionResponseChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponseChoice {
    delta: Option<ChatCompletionMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionMessage {
    role: Option<String>,
    content: Option<String>,
}

//TODO: Should refactor to move this out of openai code.
#[derive(Debug, Serialize, Deserialize)]
pub struct NativePlantEntry {
    pub common: String,
    pub scientific: String,
    pub bloom: String,
    pub description: String,
    pub image_url: Option<String>,
}

pub async fn stream_entries(
    api_key: &str,
    zip: &str,
    shade: &str,
    moisture: &str,
) -> impl Stream<Item = NativePlantEntry> {
    let prompt = build_prompt(zip, shade, moisture);
    println!("Sending prompt: {}", prompt);

    let payload = ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        messages: vec![
            ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from("You are a helpful assistant")),
            },
            ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(prompt),
            },
        ],
        max_tokens: 3000,
        stream: true,
        temperature: 0.5,
    };

    let response = call_model_stream(payload, api_key).await;

    let accumulated = Arc::new(Mutex::new(String::new()));
    let obj_start = Arc::new(Mutex::new(None));

    // I'm making two simplifying assumptions based on my use case here:
    // 1) A single chunk cannot finish two JSON objects
    // 2) Each object is flat (no nested braces)
    response.filter_map(move |chunk| {
        let accumulated = Arc::clone(&accumulated);
        let obj_start = Arc::clone(&obj_start);

        async move {
            let mut created_plant = None;
            for c in chunk.chars() {
                let mut accumulated = accumulated.lock().unwrap();
                let mut obj_start = obj_start.lock().unwrap();
                accumulated.push(c);
                match c {
                    '{' => {
                        // This character was just pushed, so its the last char of accumulated.
                        *obj_start = Some(accumulated.len() - 1);
                    }
                    '}' => {
                        let obj_end = accumulated.len();

                        let json_object = &accumulated[obj_start.unwrap()..obj_end];

                        println!("Parsing: {}", json_object);
                        if let Ok(plant) = serde_json::from_str(json_object) {
                            created_plant = plant;
                        }

                        *obj_start = None;
                    }
                    _ => {}
                }
            }
            created_plant
        }
    })
}

fn build_prompt(zip: &str, shade: &str, moisture: &str) -> String {
    format!(
        r#"You are a knowledgeable gardener living near zip code {}.
---
First, choose native plants to plant in your garden.
Next, filter to plants that will thrive in {}.
Next, filter to plants that will thrive in {}.
Next, filter to plants that support caterpillars and pollinators.

Finally, choose the top ten to plant in your garden.
---
No prose.  Your entire response will be formatted like:
```
[
  {{
    "common": "common name",
    "scientific": "scientific name",
    "bloom": "season of bloom",
    "description": "Energetically describe the wildlife it supports."
  }}
]
```"#,
        zip, shade, moisture
    )
}

async fn call_model_stream(
    payload: ChatCompletionRequest,
    api_key: &str,
) -> impl Stream<Item = String> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .expect("Error calling model");

    let status = response.status();
    if status != StatusCode::OK {
        let response_body = response
            .text()
            .await
            .expect("Can't extract text from error body");

        //TODO: Return a Result<Stream>
        panic!("Error from model endpoint: {response_body}");
    }

    let body = response
        .bytes_stream()
        .map_err(|err| -> std::io::Error { std::io::Error::new(std::io::ErrorKind::Other, err) });
    let async_read = StreamReader::new(body);

    let reader = tokio::io::BufReader::new(async_read);

    let lines = reader.lines();

    // LinesStream was the magic!
    let lines_stream = tokio_stream::wrappers::LinesStream::new(lines);

    lines_stream
        .filter_map(|line_result| async move {
            match line_result {
                Ok(line) => {
                    if line.starts_with("data: {") {
                        Some(line)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
        //        .map(|line| line.unwrap())
        .map(|line| String::from(&line[6..line.len()]))
        .map(|json| {
            let parsed: serde_json::Result<ChatCompletionResponse> = serde_json::from_str(&json);

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
        })
}
