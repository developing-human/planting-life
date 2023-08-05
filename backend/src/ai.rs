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
use async_trait::async_trait;
use futures::Stream;

pub mod openai;
mod prompts;

#[async_trait]
pub trait Ai {
    // Returns a Stream of Plants after calling openai.
    async fn stream_plants(
        &self,
        region_name: &str,
        shade: &str,
        moisture: &str,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Plant> + Send + Sync>>>;

    async fn fetch_pollinator_rating(&self, name: &str) -> anyhow::Result<u8>;
    async fn fetch_bird_rating(&self, name: &str) -> anyhow::Result<u8>;
    async fn fetch_animal_rating(&self, name: &str) -> anyhow::Result<u8>;
    async fn fetch_height(&self, name: &str) -> anyhow::Result<String>;
    async fn fetch_spread(&self, name: &str) -> anyhow::Result<String>;
    async fn fetch_bloom(&self, name: &str) -> anyhow::Result<String>;
    async fn fetch_spread_rating(&self, name: &str) -> anyhow::Result<u8>;
    async fn fetch_deer_resistance_rating(&self, name: &str) -> anyhow::Result<u8>;
    async fn fetch_conditions(&self, name: &str) -> anyhow::Result<Conditions>;
}

pub struct RealAi {
    pub open_ai: OpenAI,
}

#[async_trait]
impl Ai for RealAi {
    // Returns a Stream of Plants after calling openai.
    async fn stream_plants(
        &self,
        region_name: &str,
        shade: &str,
        moisture: &str,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = Plant> + Send + Sync>>> {
        let prompt = ListPlantsPrompt::new(region_name, shade, moisture);

        let raw_response_stream = self
            .open_ai
            .call_model_stream(prompt.build_payload(), 20000, true)
            .await?;

        ListPlantsPrompt::parse_plant_stream(raw_response_stream).await
    }

    async fn fetch_pollinator_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Pollinator))
            .await
    }

    async fn fetch_bird_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Bird))
            .await
    }

    async fn fetch_animal_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Animal))
            .await
    }

    async fn fetch_height(&self, name: &str) -> anyhow::Result<String> {
        self.open_ai
            .execute(DetailPrompt::new(name, DetailType::Height))
            .await
    }

    async fn fetch_spread(&self, name: &str) -> anyhow::Result<String> {
        self.open_ai
            .execute(DetailPrompt::new(name, DetailType::Spread))
            .await
    }

    async fn fetch_bloom(&self, name: &str) -> anyhow::Result<String> {
        self.open_ai
            .execute(DetailPrompt::new(name, DetailType::Bloom))
            .await
    }

    async fn fetch_spread_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::Spread))
            .await
    }

    async fn fetch_deer_resistance_rating(&self, name: &str) -> anyhow::Result<u8> {
        self.open_ai
            .execute(RatingPrompt::new(name, RatingType::DeerResistance))
            .await
    }

    async fn fetch_conditions(&self, name: &str) -> anyhow::Result<Conditions> {
        self.open_ai.execute(ConditionsPrompt::new(name)).await
    }
}
