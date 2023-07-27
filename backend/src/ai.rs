use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

use anyhow::anyhow;
use futures::{Stream, StreamExt};

use crate::domain::{Moisture, Plant, Rating, Shade};

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

    let response = openai::call_model_stream(payload, api_key, 2000, true).await?;

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

pub fn build_pollinator_function_prompt(name: &str) -> String {
    format!("Explain how well {} supports the pollinators of an ecosystem.  Consider its contributions as a food source, shelter, and larval host. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

Then call save_results with your explanation, a comparison to other plants, your rating, and your 30-40 word summary.", name)
}

pub fn build_bird_function_prompt(name: &str) -> String {
    format!("Explain how well {} supports the birds of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

Then call save_results with your explanation, a comparison to other plants, your rating, and your 30-40 word summary.", name)
}

pub fn build_animal_function_prompt(name: &str) -> String {
    format!("Explain how well {} supports the small ground animals of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

Then call save_results with your explanation, a comparison to other plants, your rating, and your 30-40 word summary.", name)
}

fn build_rating_formatting_instructions() -> String {
    "Next, compare how well it does compared to other plants.

Next, summarizing your findings.

Finally, rate how well it supports them on a scale from 1-10 compared to other plants.

Your entire response will be formatted as follows, the summary & rating labels are REQUIRED:
```
Your 2-4 sentence explanation.

Your 2-4 sentence comparison.

summary: Your 30-40 word summary, starting with the word 'Supports'.
rating: Your integer rating from 1-10, compared to other plants. 1-3 is average, 4-8 is for strong contributors, 9-10 is for the very best.
```

For example (the 'summary:' and 'rating:' labels are REQUIRED):
```
<plant name> are... (2-4 sentences)

Compared to other plants... (2-4 sentences)

summary: Supports... (30-40 words)
rating: 3
```
"
    .to_string()
}

pub fn build_pollinator_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports pollinators and justify your score.  To do this, lets think step by step.

First, explain how well it supports the pollinators of an ecosystem.  Consider its contributions as a food source, shelter, and larval host. If it supports specific species, mention them. Also explain how it is deficient, if applicable.

{}", name, build_rating_formatting_instructions())
}

pub fn build_bird_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports birds and justify your score.  To do this, lets think step by step.

First, explain how well it supports the birds of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

{}", name, build_rating_formatting_instructions())
}

pub fn build_animal_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports small ground animals and justify your score.  To do this, lets think step by step.

First, explain how well it supports the small ground animals of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable. 

{}", name, build_rating_formatting_instructions())
}

pub fn build_spread_rating_prompt(name: &str) -> String {
    format!("Your goal is to rate how aggressively {} spreads.  To do this, lets think step by step.

First, explain how aggressively it spreads.  Then, rate this aggressiveness on a scale from 1 to 10.

Your entire response will be formatted as follows, the rating label is REQUIRED:
```
Your 3-5 sentence description.

rating: Your integer rating from 1-10, compared to other plants. 1-4 is doesn't spread much, 5-7 is for spreading noticably, 8-10 will be very difficult to control.
```

For example:' and 'rating:' labels are REQUIRED):
```
<plant name> spreads... (2-4 sentences)

rating: 3
", name)
}

pub fn build_deer_resistance_prompt(name: &str) -> String {
    format!("Your goal is to rate the deer resistance of {}.  To do this, lets think step by step.

First, explain how it resists deer.  Then, rate this resistance on a scale from 1 to 10.

Your entire response will be formatted as follows, the rating label is REQUIRED:
```
Your 3-5 sentence description.

rating: Your integer rating from 1-10, compared to other plants. 1-4 is not deer resistant, 4-6 is not preferred by deer, 7-10 deer will not eat.
```

For example: ('rating:' label is REQUIRED):
```
<plant name> is... (3-5 sentences)

rating: 3
", name)
}

pub fn build_height_prompt(name: &str) -> String {
    format!("How tall is {}?  On the last line of your response, list only feet and inches using ' and \" for abbreviations.  Here are two examples: 

```
<plant name> typically grows to a height of 10 to 20 feet.
10'-20'
```

```
<plant name> typically grows to a height of 18 to 24 inches.
18\"-24\"
```", name)
}

pub fn build_spread_prompt(name: &str) -> String {
    format!("What is {}'s width or spread?  On the last line of your response, list only feet and inches using ' and \" for abbreviations.  Here are two examples: 

```
<plant name>'s typically spread is 10 to 20 feet.
10'-20'
```

```
<plant name>'s typically spread is 18 to 24 inches.
18\"-24\"
```", name)
}

pub fn build_bloom_prompt(name: &str) -> String {
    format!("In what season does {} typically start blooming?  Choose one of: early spring, spring, late spring, early summer, summer, late summer, early fall, fall, or late fall.  If it does not bloom, say 'does not bloom'.", name)
}

pub async fn fetch_pollinator_rating(api_key: &str, name: &str) -> anyhow::Result<Rating> {
    let prompt = build_pollinator_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 20000).await?;

    parse_rating(&response)
}

pub async fn fetch_bird_rating(api_key: &str, name: &str) -> anyhow::Result<Rating> {
    let prompt = build_bird_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 20000).await?;

    parse_rating(&response)
}

pub async fn fetch_animal_rating(api_key: &str, name: &str) -> anyhow::Result<Rating> {
    let prompt = build_animal_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 20000).await?;

    parse_rating(&response)
}

pub async fn fetch_height(api_key: &str, name: &str) -> anyhow::Result<String> {
    let prompt = build_height_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 5000).await?;

    parse_measurement(&response)
}

pub async fn fetch_spread(api_key: &str, name: &str) -> anyhow::Result<String> {
    let prompt = build_spread_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 5000).await?;

    parse_measurement(&response)
}

pub async fn fetch_bloom(api_key: &str, name: &str) -> anyhow::Result<String> {
    let prompt = build_bloom_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 5000).await?;

    parse_bloom(&response)
}

pub async fn fetch_spread_rating(api_key: &str, name: &str) -> anyhow::Result<u8> {
    let prompt = build_spread_rating_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 20000).await?;

    parse_rating_u8(&response.split('\n').collect())
}

pub async fn fetch_deer_resistance_rating(api_key: &str, name: &str) -> anyhow::Result<u8> {
    let prompt = build_deer_resistance_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 20000).await?;

    println!("deer resistance rating response: {response}");

    parse_rating_u8(&response.split('\n').collect())
}

fn build_plant_detail_request(prompt: String) -> openai::ChatCompletionRequest {
    openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        functions: vec![],
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from(
                    "You are a discerning gardener who carefully follows formatting instructions.",
                )),
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
        temperature: 0.1,
    }
}

fn parse_rating(input: &str) -> anyhow::Result<Rating> {
    let lines: Vec<&str> = input.split('\n').collect();

    let rating = parse_rating_u8(&lines)?;

    // Find the line which contains the summary
    // Sometimes it misses the "summary: " label and starts with "supports ".
    let summary_line = lines.iter().find(|line| {
        let line_lc = line.to_lowercase();

        line_lc.starts_with("summary:") || line_lc.starts_with("supports ")
    });

    // Remove the label, accounting for both upper and lower case
    // If it starts with "Supports ", nothing needs to be removed.
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

fn parse_rating_u8(lines: &Vec<&str>) -> anyhow::Result<u8> {
    // Find the line which contains the rating
    let rating_line = lines
        .iter()
        .find(|line| line.to_lowercase().starts_with("rating: "));

    // Remove the label, accounting for both upper and lower case
    let rating_str = rating_line.map(|s| s.replace("rating: ", ""));
    let rating_str = rating_str.map(|s| s.replace("Rating: ", ""));

    match rating_str.map(|line| line.parse::<u8>()) {
        Some(Ok(rating)) => Ok(rating),
        Some(Err(_)) => Err(anyhow!("invalid rating: {lines:?}",)),
        None => Err(anyhow!("rating not in response: {lines:?}")),
    }
}

/// Parses the measurement as a string out of the llm response.
/// The last line should be like: 18"-24"
fn parse_measurement(input: &str) -> anyhow::Result<String> {
    let re = Regex::new(r#"[0-9]+['"]-[0-9]+['"]"#).unwrap();
    match re.find(input).map(|m| m.as_str()) {
        Some(measurement) => Ok(measurement.to_string()),
        None => Err(anyhow!("could not find measurement in: {input}")),
    }
}

/// Parses the bloom season as a string out of the llm response.
/// Looks for the first occurance of: early spring, spring, late spring, early summer,
/// summer, late summer, early fall, fall, or late fall
fn parse_bloom(input: &str) -> anyhow::Result<String> {
    let seasons = vec![
        "early spring",
        "late spring",
        "spring",
        "early summer",
        "late summer",
        "summer",
        "early fall",
        "late fall",
        "fall",
        "early autumn",
        "late autumn",
        "autumn",
        "does not bloom",
    ];

    let input_lc = input.to_lowercase();
    for season in seasons {
        if input_lc.contains(season) {
            return Ok(season
                .replace("autumn", "fall")
                .replace("does not bloom", "N/A"));
        }
    }

    Err(anyhow!("could not find season in: {input}"))
}

fn build_prompt(region_name: &str, shade: &str, moisture: &str) -> String {
    format!(
        r#"Choose thirty plants for a new gardener's garden which are NATIVE near {}.
Their garden is in {} and {}.
Only suggest plants which do well in {} and {}.
Do NOT suggest plants which do better in other conditions.

No prose.  Your entire response will be formatted like:

scientific: Scientific Name
common: Common Name

scientific: Scientific Name
common: Common Name
"#,
        region_name,
        shade.to_uppercase(),
        moisture.to_uppercase(),
        shade.to_uppercase(),
        moisture.to_uppercase(),
    )
}

pub struct Conditions {
    pub shade: Vec<Shade>,
    pub moisture: Vec<Moisture>,
}

pub async fn fetch_conditions(api_key: &str, name: &str) -> anyhow::Result<Conditions> {
    let prompt = build_conditions_prompt(name);
    let payload = build_plant_detail_request(prompt);
    let response = openai::call_model(payload, api_key, 5000).await?;

    parse_conditions(&response)
}

fn build_conditions_prompt(name: &str) -> String {
    format!("Your goal is to answer six yes/no questions about shade and moisture conditions where {} will thrive.  First, describe growing conditions where it will thrive in 40-50 words.  

Then, use this format to answer the six questions:
- low moisture? yes/no
- medium moisture? yes/no
- high moisture? yes/no
- full shade? yes/no
- partial sun? yes/no
- full sun? yes/no
", name)
}

fn parse_conditions(input: &str) -> anyhow::Result<Conditions> {
    let input = input.to_lowercase();
    let lines: Vec<&str> = input.split('\n').collect();

    let mut conditions = Conditions {
        shade: vec![],
        moisture: vec![],
    };

    let question_moisture = vec![
        ("low moisture", Moisture::None),
        ("medium moisture", Moisture::Some),
        ("high moisture", Moisture::Lots),
    ];

    let question_shade = vec![
        ("full shade", Shade::Lots),
        ("partial sun", Shade::Some),
        ("full sun", Shade::None),
    ];

    // Matches on leading hyphens or numbered lists
    // Ex: "- ", " - ", "1. "
    let prefix_regex = Regex::new(r"[ ]*-[ ]*|\d\. *").unwrap();

    let mut answer_count = 0;
    for line in lines {
        // Sometimes GPT includes a prefix we need to remove
        let line = prefix_regex.replace(line, "");

        // I assume the response could produce "dry soil? yes" or "dry soil: yes"
        for (question, moisture) in question_moisture.iter() {
            if line.starts_with(question) {
                answer_count += 1;
                if line.contains("yes") {
                    conditions.moisture.push(*moisture)
                }
            }
        }

        for (question, shade) in question_shade.iter() {
            if line.starts_with(question) {
                answer_count += 1;
                if line.contains("yes") {
                    conditions.shade.push(*shade)
                }
            }
        }
    }

    if answer_count != 6 {
        return Err(anyhow!(
            "did not find all six condition answers in response: {input}"
        ));
    }

    if conditions.shade.is_empty() {
        return Err(anyhow!(
            "did not find any acceptable shade conditions: {input}"
        ));
    }

    if conditions.moisture.is_empty() {
        return Err(anyhow!(
            "did not find any acceptable moisture conditions: {input}"
        ));
    }

    Ok(conditions)
}

struct PlantBuilder {
    common: Option<String>,
    scientific: Option<String>,
}

impl PlantBuilder {
    fn new() -> Self {
        PlantBuilder {
            common: None,
            scientific: None,
        }
    }

    fn is_full(&self) -> bool {
        self.common.is_some() && self.scientific.is_some()
    }

    fn clear(&mut self) {
        self.common = None;
        self.scientific = None;
    }

    fn build(&self) -> Plant {
        if !self.is_full() {
            panic!("Building Plant without full builder");
        }

        Plant::new(
            &self.scientific.clone().unwrap(),
            &self.common.clone().unwrap(),
        )
    }
}

pub async fn _fetch_pollinator_rating_fn(
    api_key: &str,
    scientific_name: &str,
) -> anyhow::Result<Rating> {
    let prompt = build_pollinator_function_prompt(scientific_name);
    let payload = _build_rating_request_fn(prompt);
    let response = openai::call_model_function(payload, api_key, 20000, "save_rating").await?;

    _parse_rating_fn(&response)
}

pub async fn _fetch_bird_rating_fn(api_key: &str, scientific_name: &str) -> anyhow::Result<Rating> {
    let prompt = build_bird_function_prompt(scientific_name);
    let payload = _build_rating_request_fn(prompt);
    let response = openai::call_model_function(payload, api_key, 20000, "save_rating").await?;

    _parse_rating_fn(&response)
}

pub async fn _fetch_animal_rating_fn(
    api_key: &str,
    scientific_name: &str,
) -> anyhow::Result<Rating> {
    let prompt = build_animal_function_prompt(scientific_name);
    let payload = _build_rating_request_fn(prompt);
    let response = openai::call_model_function(payload, api_key, 20000, "save_rating").await?;

    _parse_rating_fn(&response)
}

#[derive(Deserialize)]
struct AiRating {
    summary: String,
    rating: u8,
}

fn _parse_rating_fn(input: &str) -> anyhow::Result<Rating> {
    let parsed: AiRating = serde_json::from_str(input).map_err(|e| anyhow!(e))?;

    Ok(Rating {
        reason: parsed.summary,
        rating: parsed.rating,
    })
}

fn _build_rating_function() -> ChatCompletionFunction {
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

fn _build_rating_request_fn(prompt: String) -> openai::ChatCompletionRequest {
    openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        functions: vec![_build_rating_function()],
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
