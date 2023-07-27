use futures::future::join_all;
use planting_life::{ai, domain::Rating};
use std::{
    env,
    fs::File,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let prompts_per_plant = 3;
    let plant_names = vec![
        "sunflowers",
        "bee balm",
        "mowed lawn grass",
        "blue spruce",
        "goldenrod",
        "cone flowers",
        "oak trees",
        "bamboo",
        "marigolds",
        "wild ginger",
    ];

    //TODO: Uncomment this to do a cheap/quick check
    let prompts_per_plant = 1;
    let plant_names = vec!["Asclepias syriaca"];

    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut out_file = File::create(format!("temp/prompt-eval-{}.csv", timestamp)).unwrap();
    let mut futures = vec![];

    for _ in 0..prompts_per_plant {
        for plant_name in plant_names.iter() {
            futures.push(fetch_pollinator_rating(&api_key, plant_name));
        }
    }

    let all_results = join_all(futures).await;
    let mut passing_results: Vec<(String, Rating)> = all_results
        .into_iter()
        .filter_map(|result| {
            if let Err(e) = result {
                println!("{e}");
                None
            } else {
                Some(result.unwrap())
            }
        })
        .collect();

    passing_results.sort_by(|(name1, _), (name2, _)| name1.partial_cmp(name2).unwrap());

    writeln!(out_file, r#""name","rating","reason""#).unwrap();
    for (name, rating) in passing_results {
        println!("{} {:?}", name, rating);
        writeln!(
            out_file,
            r#""{}","{}","{}""#,
            name, rating.rating, rating.reason
        )
        .unwrap();
    }

    // Write prompt at end to tie results to prompt
    writeln!(out_file).unwrap();
    writeln!(out_file).unwrap();
    writeln!(out_file).unwrap();
    let prompt = ai::build_pollinator_prompt("<name>");
    for line in prompt.split('\n') {
        // move it over a few cells to not mess up formatting
        writeln!(out_file, r#""","","{line}""#).unwrap();
    }
}

async fn fetch_pollinator_rating(
    api_key: &str,
    plant_name: &str,
) -> anyhow::Result<(String, Rating)> {
    ai::fetch_pollinator_rating(api_key, plant_name)
        .await
        .map(|rating| (plant_name.to_string(), rating))
}
