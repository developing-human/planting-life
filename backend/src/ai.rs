use serde::Deserialize;
use std::collections::HashMap;

use anyhow::anyhow;
use futures::{Stream, StreamExt};

use crate::domain::{Plant, Rating};

use self::openai::ChatCompletionFunction;

mod openai;

// Returns a Stream of Plants after calling openai.
pub async fn stream_plants(
    api_key: &str,
    region_name: &str,
    shade: &str,
    moisture: &str,
) -> anyhow::Result<impl Stream<Item = Plant>> {
    let prompt = build_prompt(region_name, shade, moisture);

    let payload = openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        functions: vec![],
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from("You are a helpful assistant")),
                function_call: None,
            },
            openai::ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(prompt),
                function_call: None,
            },
        ],
        max_tokens: 500,
        stream: true,
        temperature: 0.5,
    };

    let response = openai::call_model_stream(payload, api_key, true).await?;

    // Convert chunk stream to line stream
    // Emits Some(Some(line)) if the chunk finished a line
    // Emits Some(None) if the chunk did NOT finish a line
    // Emits None when there is nothing left to read
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

    // Convert line stream to plant stream
    // Emits Some(Some(plant)) if the line finished a plant
    // Emits Some(None) if the line did NOT finish a plant
    // Emits None when there is nothing left to read
    let plant_stream = line_stream.scan(PlantBuilder::new(), |builder, line| {
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

        // Once the builder is full, emit a built Plant
        if builder.is_full() {
            let future = futures::future::ready(Some(Some(builder.build())));
            builder.clear();

            return future;
        }

        futures::future::ready(Some(None))
    });

    // plant_stream will have None's in it from lines that did not emit an entry, remove them.
    Ok(plant_stream.filter_map(|plant| async { plant }))
}

pub fn build_pollinator_prompt(name: &str) -> String {
    format!("Explain how well {} supports the pollinators of an ecosystem.  Consider its contributions as a food source, shelter, and larval host. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

Then call save_results with your explanation, a comparison to other plants, your rating, and your 30-40 word summary.", name)
}

pub fn build_bird_prompt(name: &str) -> String {
    format!("Explain how well {} supports the birds of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

Then call save_results with your explanation, a comparison to other plants, your rating, and your 30-40 word summary.", name)
}

pub fn build_animal_prompt(name: &str) -> String {
    format!("Explain how well {} supports the small ground animals of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

Then call save_results with your explanation, a comparison to other plants, your rating, and your 30-40 word summary.", name)
}

pub async fn fetch_pollinator_rating(
    api_key: &str,
    scientific_name: &str,
) -> anyhow::Result<Rating> {
    let prompt = build_pollinator_prompt(scientific_name);
    let payload = build_rating_request(prompt);
    let response = openai::call_model_function(payload, api_key, "save_rating").await?;

    parse_rating(&response)
}

pub async fn fetch_bird_rating(api_key: &str, scientific_name: &str) -> anyhow::Result<Rating> {
    let prompt = build_bird_prompt(scientific_name);
    let payload = build_rating_request(prompt);
    let response = openai::call_model_function(payload, api_key, "save_rating").await?;

    parse_rating(&response)
}

pub async fn fetch_animal_rating(api_key: &str, scientific_name: &str) -> anyhow::Result<Rating> {
    let prompt = build_animal_prompt(scientific_name);
    let payload = build_rating_request(prompt);
    let response = openai::call_model_function(payload, api_key, "save_rating").await?;

    parse_rating(&response)
}

fn build_rating_request(prompt: String) -> openai::ChatCompletionRequest {
    openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        functions: vec![build_rating_function()],
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from("You are a discerning gardener")),
                function_call: None,
            },
            openai::ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(prompt),
                function_call: None,
            },
        ],
        max_tokens: 750,
        stream: false,
        temperature: 0.3,
    }
}

#[derive(Deserialize)]
struct AiRating {
    summary: String,
    rating: u8,
}

fn parse_rating(input: &str) -> anyhow::Result<Rating> {
    let parsed: AiRating = serde_json::from_str(input).map_err(|e| anyhow!(e))?;

    Ok(Rating {
        reason: parsed.summary,
        rating: parsed.rating,
    })
}

fn build_rating_function() -> ChatCompletionFunction {
    let mut properties = HashMap::new();
    properties.insert(
        "explanation".to_string(),
        openai::ChatCompletionProperty {
            r#type: "string".to_string(),
            description: "3-4 sentence explanation of the plant's strengths and weaknesses"
                .to_string(),
        },
    );
    properties.insert(
        "comparison".to_string(),
        openai::ChatCompletionProperty {
            r#type: "string".to_string(),
            description: "3-4 sentence comparison of the plant's contribution to other plants"
                .to_string(),
        },
    );
    properties.insert(
        "rating".to_string(),
        openai::ChatCompletionProperty {
            r#type: "integer".to_string(),
            description: "REQUIRED: an integer rating from 1-10. 1-3 is suboptimal, 4-7 is for solid contributors, 8-10 is for the very best".to_string(),
        },
    );
    properties.insert(
        "summary".to_string(),
        openai::ChatCompletionProperty {
            r#type: "string".to_string(),
            description: "30-40 word summary of the explanation and comparison".to_string(),
        },
    );

    openai::ChatCompletionFunction {
        name: "save_rating".to_string(),
        parameters: openai::ChatCompletionParameters {
            r#type: "object".to_string(),
            properties,
        },
        required: vec![
            "explanation".to_string(),
            "comparison".to_string(),
            "rating".to_string(),
            "summary".to_string(),
        ],
    }
}

fn build_prompt(region_name: &str, shade: &str, moisture: &str) -> String {
    // Requests twelve because this forms a nice grid with 1, 2, 3, or 4 columns.
    format!(
        r#"Choose twelve plants for a new gardener's garden which are NATIVE near {}.
Their garden is in {} and {}.
Only suggest plants which do well in {} and {}.
Do NOT suggest plants which do better in other conditions.

No prose.  Your entire response will be formatted like:

scientific: Scientific Name
common: Common Name
bloom: season of bloom

scientific: Scientific Name
common: Common Name
bloom: season of bloom
"#,
        region_name,
        shade.to_uppercase(),
        moisture.to_uppercase(),
        shade.to_uppercase(),
        moisture.to_uppercase(),
    )
}

struct PlantBuilder {
    common: Option<String>,
    scientific: Option<String>,
    bloom: Option<String>,
}

impl PlantBuilder {
    fn new() -> Self {
        PlantBuilder {
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

    fn build(&self) -> Plant {
        if !self.is_full() {
            panic!("Building Plant without full builder");
        }

        Plant {
            id: None,
            common: self.common.clone().unwrap(),
            scientific: self.scientific.clone().unwrap(),
            bloom: PlantBuilder::sanitize_bloom(self.bloom.clone()),
            pollinator_rating: None,
            bird_rating: None,
            animal_rating: None,
            image: None,
            usda_source: None,
            wiki_source: None,
        }
    }

    fn sanitize_bloom(bloom: Option<String>) -> Option<String> {
        if let Some(bloom) = bloom {
            for season in &["spring", "summer", "fall", "autumn", "winter"] {
                // Only use the "bloom" if it contains a season name
                // Sometimes we get values like "n/a" or "does not bloom"
                if bloom.contains(season) {
                    return Some(bloom);
                }
            }
        }

        None
    }
}
