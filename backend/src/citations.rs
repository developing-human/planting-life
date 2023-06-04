use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Citation {
    pub label: &'static str,
    pub url: String,
}

mod wikipedia {
    use reqwest::StatusCode;
    use std::time::Duration;
    use tracing::warn;

    const LABEL: &str = "Wikipedia";
    const BASE_URL: &str = "https://en.wikipedia.org/wiki/";

    pub async fn find(scientific_name: &str) -> Option<super::Citation> {
        let url = build_url(scientific_name);

        if let Some(url) = url {
            if is_valid(&url).await {
                return Some(super::Citation { label: LABEL, url });
            }
        }

        None
    }

    // Converts "Foo Bar" to "https://en.wikipedia.org/wiki/Foo_bar"
    fn build_url(scientific_name: &str) -> Option<String> {
        let scientific_name = scientific_name.to_lowercase();
        let words = scientific_name.split(' ').collect::<Vec<_>>();

        if words.len() <= 1 {
            return None;
        }

        Some(format!("{BASE_URL}{}_{}", capitalize(words[0]), words[1]))
    }

    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    #[tracing::instrument]
    async fn is_valid(url: &str) -> bool {
        let client = reqwest::Client::new();

        let response = client
            .get(url)
            .timeout(Duration::from_millis(1_000))
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(_) => {
                warn!("Error checking url: {url}");
                return false;
            }
        };

        response.status() == StatusCode::OK
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_capitalize() {
            assert_eq!(capitalize("foo"), "Foo");
            assert_eq!(capitalize("Foo"), "Foo");
        }

        #[test]
        fn test_build_url() {
            assert_eq!(
                build_url("foo bar"),
                Some("https://en.wikipedia.org/wiki/Foo_bar".to_string())
            );

            assert_eq!(
                build_url("Foo Bar"),
                Some("https://en.wikipedia.org/wiki/Foo_bar".to_string())
            );

            assert_eq!(
                build_url("Foo Bar Baz"),
                Some("https://en.wikipedia.org/wiki/Foo_bar".to_string())
            );

            assert_eq!(build_url("Foo"), None);
        }
    }
}

mod usda {
    use std::fs::File;
    use std::io::BufReader;

    use lazy_static::lazy_static;

    lazy_static! {
        static ref HASHMAP: std::collections::HashMap<String, String> = {
            let file = match File::open("resources/usda_symbols.json") {
                Ok(file) => file,
                Err(e) => panic!("Cannot load usda_symbols.json {}", e),
            };

            match serde_json::from_reader(BufReader::new(file)) {
                Ok(map) => map,
                Err(e) => panic!("Cannot deserialize usda_symbols.json {}", e),
            }
        };
    }

    const BASE_URL: &str = "https://plants.usda.gov/home/plantProfile?symbol=";

    pub fn find(scientific_name: &str) -> Option<super::Citation> {
        if let Some(symbol) = lookup_symbol(scientific_name) {
            return Some(super::Citation {
                label: "USDA",
                url: build_url(symbol),
            });
        }

        None
    }

    fn lookup_symbol(scientific_name: &str) -> Option<&String> {
        if let Some(symbol) = HASHMAP.get(&scientific_name.to_lowercase()) {
            return Some(symbol);
        }

        None
    }

    fn build_url(symbol: &str) -> String {
        format!("{BASE_URL}{symbol}")
    }
}

pub async fn find(scientific_name: &str) -> Vec<Citation> {
    let mut citations = vec![];

    if let Some(citation) = wikipedia::find(scientific_name).await {
        citations.push(citation);
    }

    if let Some(citation) = usda::find(scientific_name) {
        citations.push(citation);
    }

    citations
}
