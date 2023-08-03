use super::Prompt;
use crate::ai::openai;
use anyhow::anyhow;

pub struct RatingPrompt {
    scientific_name: String,
    rating_type: RatingType,
}

pub enum RatingType {
    Pollinator,
    Bird,
    Animal,
    Spread,
    DeerResistance,
}

impl RatingPrompt {
    pub fn new(scientific_name: &str, rating_type: RatingType) -> RatingPrompt {
        RatingPrompt {
            scientific_name: scientific_name.to_string(),
            rating_type,
        }
    }
}

impl RatingType {
    fn build_prompt_text(&self, scientific_name: &str) -> String {
        match &self {
            RatingType::Pollinator => build_pollinator_prompt(scientific_name),
            RatingType::Bird => build_bird_prompt(scientific_name),
            RatingType::Animal => build_animal_prompt(scientific_name),
            RatingType::Spread => build_spread_rating_prompt(scientific_name),
            RatingType::DeerResistance => build_deer_resistance_prompt(scientific_name),
        }
    }
}

impl Prompt for RatingPrompt {
    type Response = u8;

    fn build_payload(&self) -> openai::ChatCompletionRequest {
        let text = self.rating_type.build_prompt_text(&self.scientific_name);
        super::build_plant_detail_request(text)
    }

    fn parse_response(&self, raw_response: String) -> anyhow::Result<u8> {
        let lines: Vec<&str> = raw_response.split('\n').collect();

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
}

const RATING_FORMAT_INSTRUCTIONS: &str = "Next, compare how well it does compared to other plants. 

Next, summarizing your findings.

Finally, rate how well it supports them on a scale from 
1-10 compared to other plants.

Your entire response will be formatted as follows, the 
summary & rating labels are REQUIRED:
```
Your 2-4 sentence explanation.

Your 2-4 sentence comparison.

summary: Your 30-40 word summary, starting with the word 'Supports'.
rating: Your integer rating from 1-10, compared to other \
plants. 1-3 is average, 4-8 is for strong contributors, \
9-10 is for the very best.
```

For example (the 'summary:' and 'rating:' labels are REQUIRED):
```
<plant name> are... (2-4 sentences)

Compared to other plants... (2-4 sentences)

summary: Supports... (30-40 words)
rating: 3
```";

pub fn build_pollinator_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports pollinators and justify your score.  To do this, lets think step by step.

First, explain how well it supports the pollinators of an ecosystem.  Consider its contributions as a food source, shelter, and larval host. If it supports specific species, mention them. Also explain how it is deficient, if applicable.

{}", name, RATING_FORMAT_INSTRUCTIONS)
}

pub fn build_bird_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports birds and justify your score.  To do this, lets think step by step.

First, explain how well it supports the birds of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable.

{}", name, RATING_FORMAT_INSTRUCTIONS)
}

pub fn build_animal_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports small ground animals and justify your score.  To do this, lets think step by step.

First, explain how well it supports the small ground animals of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable.

{}", name, RATING_FORMAT_INSTRUCTIONS)
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
