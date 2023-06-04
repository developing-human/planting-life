use csv::Reader;
use std::collections::HashMap;

struct Entry {
    symbol: String,
    original_length: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let raw_csv_data = fetch_original_data().await;

    // This map ends up mapping a lowercase two word scientific name to a symbol
    // Each entry also holds the original length of the scientific name as the
    // longer names tend to be variations of the original plant.
    let mut name_to_symbol: HashMap<String, Entry> = HashMap::new();

    let mut reader = Reader::from_reader(raw_csv_data.as_bytes());
    for record in reader.records() {
        let record = record.unwrap();
        let symbol = record.get(0).expect("Should have column 0");
        let scientific_name = record.get(2).expect("Should have column 2").to_lowercase();

        let first_two_words = scientific_name.split(' ').collect::<Vec<_>>()[0..2].join(" ");

        // Keep this entry, unless the scientific name is longer.
        let mut should_update = true;
        if let Some(entry) = name_to_symbol.get(&first_two_words) {
            if scientific_name.len() >= entry.original_length {
                should_update = false;
            }
        }

        if should_update {
            name_to_symbol.insert(
                first_two_words,
                Entry {
                    symbol: String::from(symbol),
                    original_length: scientific_name.len(),
                },
            );
        }
    }

    // At this point, the map has scientific name -> symbol + extra info
    // We want to output the name -> symbol, without extra info
    let name_to_symbol: HashMap<String, String> = name_to_symbol
        .iter()
        .map(|(k, v)| (k.clone(), v.symbol.clone()))
        .collect();

    // I think readability is worth the extra file size, since these aren't heavily indented
    let serialized = serde_json::to_string_pretty(&name_to_symbol).unwrap();

    println!("{}", serialized);
}

async fn fetch_original_data() -> String {
    let client = reqwest::Client::new();

    let response = client
        .get("https://plants.usda.gov/assets/docs/CompletePLANTSList/plantlst.txt")
        .send()
        .await
        .expect("Requesting data should succeed");

    response
        .text()
        .await
        .expect("Should receive requested data")
}
