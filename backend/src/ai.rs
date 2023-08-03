use self::prompts::{
    conditions::ConditionsPrompt,
    details::{DetailPrompt, DetailType},
    list::{self},
    ratings::{RatingPrompt, RatingType},
};
use crate::domain::{Conditions, Plant};
use futures::Stream;

mod openai;
mod prompts;

// Returns a Stream of Plants after calling openai.
pub async fn stream_plants(
    api_key: &str,
    region_name: &str,
    shade: &str,
    moisture: &str,
) -> anyhow::Result<impl Stream<Item = Plant> + Send> {
    let payload = list::build_payload(region_name, shade, moisture);
    let response = openai::call_model_stream(payload, api_key, 20000, true).await?;
    list::parse_plant_stream(response).await
}

pub async fn fetch_pollinator_rating(api_key: &str, name: &str) -> anyhow::Result<u8> {
    prompts::execute(RatingPrompt::new(name, RatingType::Pollinator), api_key).await
}

pub async fn fetch_bird_rating(api_key: &str, name: &str) -> anyhow::Result<u8> {
    prompts::execute(RatingPrompt::new(name, RatingType::Bird), api_key).await
}

pub async fn fetch_animal_rating(api_key: &str, name: &str) -> anyhow::Result<u8> {
    prompts::execute(RatingPrompt::new(name, RatingType::Animal), api_key).await
}

pub async fn fetch_height(api_key: &str, name: &str) -> anyhow::Result<String> {
    prompts::execute(DetailPrompt::new(name, DetailType::Height), api_key).await
}

pub async fn fetch_spread(api_key: &str, name: &str) -> anyhow::Result<String> {
    prompts::execute(DetailPrompt::new(name, DetailType::Spread), api_key).await
}

pub async fn fetch_bloom(api_key: &str, name: &str) -> anyhow::Result<String> {
    prompts::execute(DetailPrompt::new(name, DetailType::Bloom), api_key).await
}

pub async fn fetch_spread_rating(api_key: &str, name: &str) -> anyhow::Result<u8> {
    prompts::execute(RatingPrompt::new(name, RatingType::Spread), api_key).await
}

pub async fn fetch_deer_resistance_rating(api_key: &str, name: &str) -> anyhow::Result<u8> {
    prompts::execute(RatingPrompt::new(name, RatingType::DeerResistance), api_key).await
}

pub async fn fetch_conditions(api_key: &str, name: &str) -> anyhow::Result<Conditions> {
    prompts::execute(ConditionsPrompt::new(name), api_key).await
}
