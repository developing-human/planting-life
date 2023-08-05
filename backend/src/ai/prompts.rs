use crate::ai::openai;
use futures::Stream;

pub mod conditions;
pub mod details;
pub mod list;
pub mod ratings;

/// A prompt knows how to build a payload for an openai request
/// and how to parse the response.
pub trait Prompt {
    type Response;

    fn build_payload(&self) -> openai::ChatCompletionRequest;
    fn parse_response(&self, raw_response: String) -> anyhow::Result<Self::Response>;
}

pub trait StreamingPrompt {
    type Response;

    fn build_payload(&self) -> openai::ChatCompletionRequest;
    fn parse_response(
        &self,
        raw_response: impl Stream<Item = String>,
    ) -> anyhow::Result<Box<dyn Stream<Item = Self::Response> + Unpin + Send>>;
}

fn build_plant_detail_request(prompt: String) -> openai::ChatCompletionRequest {
    openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from(
                    "You are a discerning gardener who carefully follows formatting instructions.",
                )),
            },
            openai::ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(prompt),
            },
        ],
        max_tokens: 750,
        stream: false,
        temperature: 0.1,
    }
}
