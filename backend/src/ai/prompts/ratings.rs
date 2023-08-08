use super::Prompt;
use crate::ai::openai;
use anyhow::anyhow;
use regex::Regex;

pub struct RatingPrompt {
    scientific_name: String,
    rating_type: RatingType,
}

pub enum RatingType {
    Pollinator,
    Bird,
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

    fn parse_response(&self, raw_response: &str) -> anyhow::Result<u8> {
        let lines: Vec<&str> = raw_response.split('\n').collect();

        // First, try to find the structured rating, as this is what
        // is usually returned.
        // Find the line which contains "rating:"
        let rating_line = lines
            .iter()
            .find(|line| line.to_lowercase().contains("rating:"));

        // There could be spaces before rating or after the colon.
        // Split on the colon and trim any whitespace.
        let rating_split: Option<Vec<&str>> = rating_line.map(|l| l.split(':').collect());
        let rating_str = rating_split.map(|splt| splt[1].trim());

        if let Some(Ok(rating)) = rating_str.map(|line| line.parse::<u8>()) {
            return Ok(rating);
        }

        // Find any matching regexes which can capture the rating.  Try to parse it.
        // As the LLM finds more creative ways to not follow instructions, add to
        // this list.
        let unstructured_regexes = vec![
            r"(\d) out of 10",
            r"(\d) on a scale from 1 to 10",
            r"(\d)/10",
            r"rate [a-zA-Z ]* as a (\d)",
        ];
        for unstructured_regex in unstructured_regexes {
            let regex = Regex::new(unstructured_regex).unwrap();
            for (_, [rating_str]) in regex.captures_iter(raw_response).map(|c| c.extract()) {
                if let Ok(rating) = rating_str.parse::<u8>() {
                    return Ok(rating);
                }
            }
        }

        Err(anyhow!(
            "could not parse rating from response: {raw_response:?}"
        ))
    }
}

const RATING_FORMAT_INSTRUCTIONS: &str = "Next, compare how well it does compared to other plants. 

Finally, rate how well it supports them on a scale from 
1-10 compared to other plants.

Your entire response will be formatted as follows, the 
'rating:' label is REQUIRED:
```
Your 2-4 sentence explanation.

Your 2-4 sentence comparison.

rating: Your integer rating from 1-10, compared to other \
plants. 1-3 is average, 4-8 is for strong contributors, \
9-10 is for the very best.
```

For example (the 'rating:' label is REQUIRED):
```
<plant name> are... (2-4 sentences)

Compared to other plants... (2-4 sentences)

rating: 3
```";

pub fn build_pollinator_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports pollinators.  To do this, lets think step by step.

First, explain how well it supports the pollinators of an ecosystem.  Consider its contributions as a food source, shelter, and larval host. If it supports specific species, mention them. Also explain how it is deficient, if applicable.

{}", name, RATING_FORMAT_INSTRUCTIONS)
}

pub fn build_bird_prompt(name: &str) -> String {
    format!("Your goal is to rate {} compared to other plants with respect to how well it supports birds.  To do this, lets think step by step.

First, explain how well it supports the birds of an ecosystem.  Consider its contributions as a food source, shelter, and nesting site. If it supports specific species, mention them. Also explain how it is deficient, if applicable.

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

For example ('rating:' label is REQUIRED):
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase_labeled_rating() {
        let rating = parse_rating(
            "blah blah blah
rating: 8
blah blah blah",
        );

        assert_eq!(rating.unwrap(), 8)
    }

    #[test]
    fn test_lowercase_labeled_rating_with_indent() {
        let rating = parse_rating(
            "blah blah blah
            rating: 8
            blah blah blah",
        );

        assert_eq!(rating.unwrap(), 8)
    }

    #[test]
    fn test_uppercase_labeled_rating() {
        let rating = parse_rating(
            "blah blah blah
Rating: 8
blah blah blah",
        );

        assert_eq!(rating.unwrap(), 8)
    }

    #[test]
    fn test_uppercase_labeled_rating_no_space() {
        let rating = parse_rating(
            "blah blah blah
Rating:8
blah blah blah",
        );

        assert_eq!(rating.unwrap(), 8)
    }

    #[test]
    fn test_unlabeled_rating() {
        let rating = parse_rating(
            "Threadleaf coreopsis is a...

In terms of ...can be rated as a 7 on a scale from 1 to 10. While it is not...",
        );

        assert_eq!(rating.unwrap(), 7)
    }

    #[test]
    fn test_unlabeled_rating_2() {
        let rating = parse_rating(
            "Threadleaf coreopsis is a...

In terms of ...can be rated 7 out of 10. While it is not...",
        );

        assert_eq!(rating.unwrap(), 7)
    }

    #[test]
    fn test_unlabeled_rating_3() {
        let rating = parse_rating(
            "Threadleaf coreopsis is a...

In terms of ...can be rated 7/10. While it is not...",
        );

        assert_eq!(rating.unwrap(), 7)
    }

    #[test]
    fn test_unlabeled_rating_4() {
        let rating = parse_rating(
            "Threadleaf coreopsis is a...

I would rate blah blah as a 6. While it is not...",
        );

        assert_eq!(rating.unwrap(), 6)
    }

    fn parse_rating(ai_response: &str) -> anyhow::Result<u8> {
        let prompt = RatingPrompt::new("name", RatingType::Bird);
        prompt.parse_response(ai_response)
    }
}
