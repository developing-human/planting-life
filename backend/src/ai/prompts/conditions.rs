use anyhow::anyhow;
use regex::Regex;

use crate::{
    ai::openai,
    domain::{Conditions, Moisture, Shade},
};

use super::Prompt;

pub struct ConditionsPrompt {
    scientific_name: String,
}

impl ConditionsPrompt {
    pub fn new(scientific_name: &str) -> Self {
        Self {
            scientific_name: scientific_name.to_string(),
        }
    }
}

impl Prompt for ConditionsPrompt {
    type Response = Conditions;

    fn build_payload(&self) -> openai::ChatCompletionRequest {
        let text = build_conditions_prompt(&self.scientific_name);
        super::build_plant_detail_request(text)
    }

    fn parse_response(&self, raw_response: String) -> anyhow::Result<Conditions> {
        parse_conditions(&raw_response)
    }
}

fn build_conditions_prompt(name: &str) -> String {
    format!(
        "Your goal is to answer six yes/no questions about shade \
         and moisture conditions where {} will thrive.  First, describe \
         growing conditions where it will thrive in 40-50 words.

         Then, use this format to answer the six questions: \n\
         - low moisture? yes/no \n\
         - medium moisture? yes/no \n\
         - high moisture? yes/no \n\
         - full shade? yes/no \n\
         - partial sun? yes/no \n\
         - full sun? yes/no",
        name
    )
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
