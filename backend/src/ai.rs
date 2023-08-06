use std::pin::Pin;

use self::{
    openai::OpenAI,
    prompts::{
        conditions::ConditionsPrompt,
        details::{DetailPrompt, DetailType},
        list::ListPlantsPrompt,
        ratings::{RatingPrompt, RatingType},
    },
};
use crate::domain::{Conditions, Plant};
use futures::Stream;
use mockall::automock;

pub mod openai;
mod prompts;

pub struct Ai {
    pub open_ai: OpenAI,
}

#[automock]
impl Ai {
    pub fn new(open_ai: OpenAI) -> Self {
        Self { open_ai }
    }

    // Returns a Stream of Plants after calling openai.
    pub async fn stream_plants(
        &self,
        region_name: &str,
        shade: &str,
        moisture: &str,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Plant> + Send>>> {
        let prompt = ListPlantsPrompt::new(region_name, shade, moisture);

        let raw_response_stream = self
            .open_ai
            .call_model_stream(prompt.build_payload(), 20000, true)
            .await?;

        ListPlantsPrompt::parse_plant_stream(raw_response_stream).await
    }

    pub async fn fetch_pollinator_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Pollinator))
            .await
    }

    pub async fn fetch_bird_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Bird))
            .await
    }

    pub async fn fetch_animal_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Animal))
            .await
    }

    pub async fn fetch_height(&self, name: &str) -> anyhow::Result<String> {
        self.open_ai
            .execute(DetailPrompt::new(name, DetailType::Height))
            .await
    }

    pub async fn fetch_spread(&self, name: &str) -> anyhow::Result<String> {
        self.open_ai
            .execute(DetailPrompt::new(name, DetailType::Spread))
            .await
    }

    pub async fn fetch_bloom(&self, name: &str) -> anyhow::Result<String> {
        self.open_ai
            .execute(DetailPrompt::new(name, DetailType::Bloom))
            .await
    }

    pub async fn fetch_spread_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Spread))
            .await
    }

    pub async fn fetch_deer_resistance_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::DeerResistance))
            .await
    }

    pub async fn fetch_conditions(&self, name: &str) -> anyhow::Result<Conditions> {
        self.open_ai.execute(ConditionsPrompt::new(name)).await
    }
}
