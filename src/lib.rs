use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};

#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionResponseChoice>,
}

#[derive(Debug, Deserialize)]
struct CompletionResponseChoice {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativePlantEntry {
    pub common: String,
    pub scientific: String,
    pub description: String,
    pub image_url: Option<String>,
}

pub fn fetch_entries(
    api_key: &str,
    zip: &str,
    shade: &str,
    moisture: &str,
) -> Vec<NativePlantEntry> {
    let payload = CompletionRequest {
        model: String::from("text-davinci-003"),
        prompt: build_prompt(zip, shade, moisture),
        max_tokens: 4000,
        stream: false,
    };

    let response = call_model(payload, api_key);
    let parsed_response = &response.choices.get(0).unwrap().text;

    serde_json::from_str(parsed_response).expect("Error parsing inner response")
}

pub fn stream_entries(
    api_key: &str,
    zip: &str,
    shade: &str,
    moisture: &str,
) -> impl Iterator<Item = NativePlantEntry> {
    let payload = CompletionRequest {
        model: String::from("text-davinci-003"),
        prompt: build_prompt(zip, shade, moisture),
        max_tokens: 4000,
        stream: true,
    };

    let response = call_model_stream(payload, api_key);

    let mut accumulated = String::new();
    let mut obj_start = None;

    // I'm making two simplifying assumptions based on my use case here:
    // 1) A single chunk cannot contain an entire JSON object.
    // 2) Each object is flat (no nested braces)
    response.filter_map(move |chunk| {
        let mut created_object = None;
        for c in chunk.chars() {
            accumulated.push(c);
            match c {
                '{' => {
                    // This character was just pushed, so its the last char of accumulated.
                    obj_start = Some(accumulated.len() - 1);
                }
                '}' => {
                    let obj_end = accumulated.len();

                    let one_object = &accumulated[obj_start.unwrap()..obj_end];
                    println!("Parsing: {}", one_object);
                    created_object =
                        Some(serde_json::from_str::<NativePlantEntry>(one_object).unwrap());

                    obj_start = None;
                }
                _ => {}
            }
        }

        created_object
    })
}

fn build_prompt(zip: &str, shade: &str, moisture: &str) -> String {
    format!(
        r#"[no prose][respond with minified JSON] Suggest 5 plants to plant near zip code {} which would do well in {} and {}.  All suggestions must be native to that area.  Respond like: [ {{ "common": "the common name", "scientific": "the scientific name", "description": "a description of the plant, 25 words or less" }} ]"#,
        zip, shade, moisture
    )
}

fn call_model_stream(payload: CompletionRequest, api_key: &str) -> impl Iterator<Item = String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.openai.com/v1/completions")
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

    println!("Creating bufreader");
    let reader = io::BufReader::new(response);
    //for line in reader.lines() {
    //    println!("{}", line.unwrap());
    //}

    println!("Created bufreader");
    reader
        .lines()
        .filter(|line_result| match line_result {
            Ok(line) => line.starts_with("data: {"),
            Err(_) => false,
        })
        .map(|line| line.unwrap())
        .map(|line| String::from(&line[6..line.len()]))
        .map(|json| {
            let parsed: serde_json::Result<CompletionResponse> = serde_json::from_str(&json);

            parsed.expect("Error parsing inner response")
        })
        .map(|parsed_response| String::from(&parsed_response.choices.get(0).unwrap().text))

    //vec![String::from("foo")].into_iter()
}

fn call_model(payload: CompletionRequest, api_key: &str) -> CompletionResponse {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.openai.com/v1/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .expect("Error calling model");

    let status = response.status();
    let response_body = response
        .text()
        .expect("Error extracting body from response");

    if status != StatusCode::OK {
        eprintln!("Error from model endpoint: {response_body}");
        std::process::exit(1);
    }

    serde_json::from_str(&response_body).expect("Error parsing response")
}
