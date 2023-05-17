use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};

#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    stream: bool,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionResponseChoice>,
}

#[derive(Debug, Deserialize)]
struct CompletionResponseChoice {
    text: String,
}

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
    message: Option<ChatCompletionMessage>,
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

//TODO: I think this should stream chunks back.  And trickle parser
//      should turn chunks into events.  Then events go to front end.
pub fn stream_entries(
    api_key: &str,
    zip: &str,
    shade: &str,
    moisture: &str,
) -> impl Iterator<Item = NativePlantEntry> {
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
        temperature: 0.4,
    };

    let response = call_model_stream(payload, api_key);

    let mut accumulated = String::new();
    let mut obj_start = None;

    // I'm making two simplifying assumptions based on my use case here:
    // 1) A single chunk cannot finish two JSON objects
    // 2) Each object is flat (no nested braces)
    response.filter_map(move |chunk| {
        let mut created_plant = None;
        for c in chunk.chars() {
            accumulated.push(c);
            match c {
                '{' => {
                    // This character was just pushed, so its the last char of accumulated.
                    obj_start = Some(accumulated.len() - 1);
                }
                '}' => {
                    let obj_end = accumulated.len();

                    let json_object = &accumulated[obj_start.unwrap()..obj_end];

                    println!("Parsing: {}", json_object);
                    if let Ok(plant) = serde_json::from_str(json_object) {
                        created_plant = plant;
                    }

                    obj_start = None;
                }
                _ => {}
            }
        }

        created_plant
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

Finally, choose the top five to plant in your garden.
---
No prose.  Your entire response will be formatted like:
```
[
  {{
    "common": "common name",
    "scientific": "scientific name",
    "bloom": "season of bloom",
    "description": "Energetically describe the wildlife it supports"
  }}
]
```"#,
        zip, shade, moisture
    )
}

fn call_model_stream(
    payload: ChatCompletionRequest,
    api_key: &str,
) -> impl Iterator<Item = String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .expect("Error calling model");

    let status = response.status();
    if status != StatusCode::OK {
        let response_body = response.text().expect("Can't extract text from error body");

        eprintln!("Error from model endpoint: {response_body}");
        std::process::exit(1);
    }

    //TODO: I can use a larger buffer by calling with_capacity.  Is this worthwhile?
    let reader = io::BufReader::new(response);

    reader
        .lines()
        .filter(|line_result| match line_result {
            Ok(line) => line.starts_with("data: {"),
            Err(_) => false,
        })
        .map(|line| line.unwrap())
        .map(|line| String::from(&line[6..line.len()]))
        .map(|json| {
            let parsed: serde_json::Result<ChatCompletionResponse> = serde_json::from_str(&json);

            parsed.expect("Error parsing inner response")
        })
        .filter_map(|parsed_response| {
            let delta = &parsed_response.choices.get(0).unwrap().delta;

            if let Some(delta) = delta {
                if let Some(content) = &delta.content {
                    return Some(String::from(content));
                }
            }

            None
        })
}
