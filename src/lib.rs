use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionResponseChoice>,
}

#[derive(Debug, Deserialize)]
struct CompletionResponseChoice {
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct NativePlantEntry {
    pub common: String,
    pub scientific: String,
    pub description: String,
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
    };

    let response = call_model(payload, api_key);
    let parsed_response = &response.choices.get(0).unwrap().text;

    serde_json::from_str(parsed_response).expect("Error parsing inner response")
}

fn build_prompt(zip: &str, shade: &str, moisture: &str) -> String {
    format!(
        r#"[no prose][respond with minified JSON] Suggest 5 plants to plant near zip code {} which would do well in {} and {}.  All suggestions must be native to that area.  Respond like: [ {{ "common": "the common name", "scientific": "the scientific name", "description": "a description of the plant, 25 words or less" }} ]"#,
        zip, shade, moisture
    )
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
