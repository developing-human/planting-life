use anyhow::anyhow;
use regex::Regex;

use crate::ai::openai;

use super::Prompt;

pub struct DetailPrompt {
    scientific_name: String,
    detail_type: DetailType,
}

pub enum DetailType {
    Height,
    Spread,
    Bloom,
}

impl DetailPrompt {
    pub fn new(scientific_name: &str, detail_type: DetailType) -> Self {
        Self {
            scientific_name: scientific_name.to_string(),
            detail_type,
        }
    }
}

impl Prompt for DetailPrompt {
    type Response = String;

    fn build_payload(&self) -> openai::ChatCompletionRequest {
        let text = match &self.detail_type {
            DetailType::Height => build_height_prompt(&self.scientific_name),
            DetailType::Spread => build_spread_prompt(&self.scientific_name),
            DetailType::Bloom => build_bloom_prompt(&self.scientific_name),
        };

        super::build_plant_detail_request(text)
    }

    fn parse_response(&self, raw_response: &str) -> anyhow::Result<String> {
        match &self.detail_type {
            DetailType::Height => parse_measurement(&raw_response),
            DetailType::Spread => parse_measurement(&raw_response),
            DetailType::Bloom => parse_bloom(&raw_response),
        }
    }
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
