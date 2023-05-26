use futures::{stream::StreamExt, TryStreamExt};

use futures::Stream;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
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
pub struct NativePlant {
    pub common: String,
    pub scientific: String,
    pub bloom: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

struct NativePlantBuilder {
    common: Option<String>,
    scientific: Option<String>,
    bloom: Option<String>,
}

impl NativePlantBuilder {
    fn new() -> Self {
        NativePlantBuilder {
            common: None,
            scientific: None,
            bloom: None,
        }
    }

    fn is_full(&self) -> bool {
        self.common.is_some() && self.scientific.is_some() && self.bloom.is_some()
    }

    fn clear(&mut self) {
        self.common = None;
        self.scientific = None;
        self.bloom = None;
    }

    fn build(&self) -> NativePlant {
        if !self.is_full() {
            panic!("Building NativePlant without full builder");
        }

        NativePlant {
            common: self.common.clone().unwrap(),
            scientific: self.scientific.clone().unwrap(),
            bloom: self.bloom.clone().unwrap(),
            description: None,
            image_url: None,
        }
    }
}

// Returns a Stream of NativePlantEntries after calling openai.
pub async fn stream_plants(
    api_key: &str,
    zip: &str,
    shade: &str,
    moisture: &str,
) -> impl Stream<Item = NativePlant> {
    let prompt = build_prompt(zip, shade, moisture);

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
        max_tokens: 500,
        stream: true,
        temperature: 0.5,
    };

    let response = call_model_stream(payload, api_key).await;

    let line_stream = response.scan(String::new(), |state, chunk| {
        state.push_str(&chunk);
        if let Some(pos) = state.find('\n') {
            // Take from start of line to end of line
            let line = state[..pos].to_owned();

            // Remove any of the returned line from state
            state.replace_range(..pos + 1, "");
            futures::future::ready(Some(Some(line)))
        } else {
            futures::future::ready(Some(None))
        }
    });

    let plant_stream = line_stream.scan(NativePlantBuilder::new(), |builder, line| {
        if line.is_none() {
            return futures::future::ready(Some(None));
        }

        let line = line.unwrap();
        if !line.contains(':') {
            return futures::future::ready(Some(None));
        }

        // Since we checked existence of ":", we know there will be at least two entries
        // and these unwraps will not panic
        let split: Vec<&str> = line.split(':').collect();
        let key = split.first().unwrap().trim();
        let value = Some(String::from(split.get(1).unwrap().trim()));

        // Store labeled values in the builder
        match key {
            "scientific" => builder.scientific = value,
            "common" => builder.common = value,
            "bloom" => builder.bloom = value,
            _ => return futures::future::ready(Some(None)),
        }

        // Once the builder is full, emit a built NativePlant
        if builder.is_full() {
            let future = futures::future::ready(Some(Some(builder.build())));
            builder.clear();

            return future;
        }

        futures::future::ready(Some(None))
    });

    // plant_stream will have None's in it from lines that did not emit an entry, remove them.
    plant_stream.filter_map(|plant| async { plant })
}

pub async fn fetch_description(api_key: &str, scientific_name: &str) -> impl Stream<Item = String> {
    let prompt = format!(
        "Describe the specific wildlife {} supports in 25-35 words by completing this sentence: 
         Supports ...",
        scientific_name
    );

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
        max_tokens: 200,
        stream: true,
        temperature: 0.5,
    };

    call_model_stream(payload, api_key).await
}

fn build_prompt(zip: &str, shade: &str, moisture: &str) -> String {
    format!(
        r#"You are a knowledgeable gardener living near zip code {}.

Choose ten plants for your garden which are NATIVE near zip code {} and will THRIVE in {} and {}.  
Prioritize plants which support pollinators.

No prose.  Your entire response will be formatted like:

scientific: Scientific Name
common: Common Name
bloom: season of bloom

scientific: Scientific Name
common: Common Name
bloom: season of bloom
"#,
        zip,
        zip,
        shade.to_uppercase(),
        moisture.to_uppercase()
    )
}

// Returns a stream of short strings from openai
// If openai as trying return "foo bar baz", one chunk could be "foo b"
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
