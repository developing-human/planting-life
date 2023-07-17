use anyhow::anyhow;
use futures::{Stream, StreamExt};

use crate::domain::{Plant, Rating};

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
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from("You are a helpful assistant")),
            },
            openai::ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(prompt),
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

pub async fn fetch_description(
    api_key: &str,
    scientific_name: &str,
) -> anyhow::Result<impl Stream<Item = String>> {
    let prompt = format!(
        "Describe the specific wildlife {} supports in 25-35 words by completing this sentence: 
         Supports ...",
        scientific_name
    );

    let payload = openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from("You are a knowledgeable gardener")),
            },
            openai::ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(prompt),
            },
        ],
        max_tokens: 200,
        stream: true,
        temperature: 0.3,
    };

    openai::call_model_stream(payload, api_key, false).await
}

pub fn build_pollinator_prompt(name: &str) -> String {
    format!("Explain how well {} supports the pollinators of an ecosystem in 3-5 sentences.  Consider it's contributions as a food source, shelter, and larval host.  If it supports 
specific species, mention them.  Also explain how it is deficient, if applicable.  

Next, write a 3-5 sentences comparing how well it supports pollinators compared to other plants.

Next, summarize your findings in ~40 words.

Finally, rate how well it supports them on a scale from 1-10 compared to other plants.  
1 is worst, 5 is average, 10 is best.


Your entire response will be formatted as follows, the labels are REQUIRED:

Your explanation.
Your comparison to other plants.

summary: ~40 words summarizing your explanation, starting with the word \"Supports\".  Do NOT mention the rating.
rating: an integer rating from 1-10

For example:
A paragraph explaining.

A paragraph comparing.

summary: Supports... <rest of ~40 word summary>
rating: 5", name)
}

pub async fn fetch_pollinator_rating(
    api_key: &str,
    scientific_name: &str,
) -> anyhow::Result<Rating> {
    let prompt = build_pollinator_prompt(scientific_name);
    let payload = build_rating_request(prompt);
    let response = openai::call_model(payload, api_key).await?;

    parse_rating(&response)
}

pub async fn fetch_bird_rating(api_key: &str, scientific_name: &str) -> anyhow::Result<Rating> {
    let prompt = format!("Summarize how {} supports the birds of an ecosystem in ~40 words.  Consider it's contributions as a food source and shelter.  If it supports specific species, mention them.  Also explain how it is deficient, if applicable.  Rate how well it supports them on a scale from 1-10 compared to other plants.  1-2 is bad, 3-5 is mediocre, 6-8 is good, 9-10 is great.
Your entire response will be formatted as follows, the labels are REQUIRED:

summary: ~40 words summarizing your explanation, starting with the word \"Supports\".  Do NOT repeat the rating.
rating: an integer rating from 1-10

For example:

summary: Supports... <rest of ~40 word summary>
rating: 5", scientific_name);

    let payload = build_rating_request(prompt);
    let response = openai::call_model(payload, api_key).await?;

    parse_rating(&response)
}

pub async fn fetch_animal_rating(api_key: &str, scientific_name: &str) -> anyhow::Result<Rating> {
    let prompt = format!("Summarize how {} supports the small ground animals (such as mammals and reptiles) of an ecosystem in ~40 words.  Consider it's contributions as a food source and shelter.  If it supports specific species, mention them.  Also explain how it is deficient, if applicable.  Rate how well it supports them on a scale from 1-10 compared to other plants.  1-2 is bad, 3-5 is mediocre, 6-8 is good, 9-10 is great.

Your entire response will be formatted as follows, the labels are REQUIRED:

summary: ~40 words summarizing your explanation, starting with the word \"Supports\".  Do NOT repeat the rating.
rating: an integer rating from 1-10

For example:

summary: Supports... rest of ~40 word summary
rating: 5
", scientific_name);

    let payload = build_rating_request(prompt);
    let response = openai::call_model(payload, api_key).await?;

    parse_rating(&response)
}

fn build_rating_request(prompt: String) -> openai::ChatCompletionRequest {
    openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from("You are a knowledgeable gardener")),
            },
            openai::ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(prompt),
            },
        ],
        max_tokens: 500,
        stream: false,
        temperature: 0.3,
    }
}

fn parse_rating(input: &str) -> anyhow::Result<Rating> {
    let lines: Vec<&str> = input.split('\n').collect();

    // Find the line which contains the rating
    let rating_line = lines
        .iter()
        .find(|line| line.to_lowercase().starts_with("rating: "));

    // Remove the label, accounting for both upper and lower case
    let rating_str = rating_line.map(|s| s.replace("rating: ", ""));
    let rating_str = rating_str.map(|s| s.replace("Rating: ", ""));

    // Parse the rating into an integer
    let rating = match rating_str.map(|line| line.parse()) {
        Some(Ok(rating)) => rating,
        Some(Err(_)) => return Err(anyhow!("invalid rating: {input}",)),
        None => return Err(anyhow!("rating not in response: {input}")),
    };

    // Find the line which contains the summary
    let summary_line = lines
        .iter()
        .find(|line| line.to_lowercase().starts_with("summary:"));

    // Remove the label, accounting for both upper and lower case
    let summary_str = summary_line.map(|s| s.replace("summary: ", ""));
    let summary_str = summary_str.map(|s| s.replace("Summary: ", ""));

    match summary_str {
        Some(summary) => Ok(Rating {
            rating,
            reason: summary,
        }),
        None => Err(anyhow!("summary not in response: {input}")),
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
            description: None,
            pollinator_rating: None,
            bird_rating: None,
            animal_rating: None,
            image: None,
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
