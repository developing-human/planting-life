use crate::{ai::openai, domain::Plant};
use futures::{Stream, StreamExt};

// This does not use a Prompt trait because traits cannot
// have async functions on them.  I kept this in the prompts
// folder just for organizational purposes.

pub fn build_payload(
    region_name: &str,
    shade: &str,
    moisture: &str,
) -> openai::ChatCompletionRequest {
    let text = build_prompt(region_name, shade, moisture);

    openai::ChatCompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        messages: vec![
            openai::ChatCompletionMessage {
                role: Some(String::from("system")),
                content: Some(String::from("You are a helpful assistant")),
            },
            openai::ChatCompletionMessage {
                role: Some(String::from("user")),
                content: Some(text),
            },
        ],
        max_tokens: 500,
        stream: true,
        temperature: 0.5,
    }
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

// Returns a Stream of Plants after calling openai.
pub async fn parse_plant_stream(
    raw_response: impl Stream<Item = String>,
) -> anyhow::Result<impl Stream<Item = Plant>> {
    // Convert chunk stream to line stream
    // Emits Some(Some(line)) if the chunk finished a line
    // Emits Some(None) if the chunk did NOT finish a line
    // Emits None when there is nothing left to read
    let line_stream = raw_response.scan(String::new(), |state, chunk| {
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
