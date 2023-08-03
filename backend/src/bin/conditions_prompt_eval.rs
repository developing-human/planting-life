use futures::future::join_all;
use planting_life::{
    ai::{self},
    domain::Conditions,
};
use std::{
    env,
    fs::File,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let prompts_per_plant = 5;
    let plant_names = vec![
        "Monarda fistulosa",
        "Asclepias incarnata",
        "Liatris spicata",
        "Vernonia noveboracensis",
        "Lobelia cardinalis",
        "Echinacea purpurea",
        "Solidago canadensis",
        "Penstemon digitalis",
        "Iris versicolor",
        "Lobelia siphilitica",
        "Rudbeckia hirta",
        "Helenium autumnale",
    ];

    //TODO: Uncomment this to do a cheap/quick check
    //let prompts_per_plant = 5;
    //let plant_names = vec!["Monarda fistulosa"];

    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut out_file = File::create(format!("temp/prompt-eval-{}.csv", timestamp)).unwrap();
    let mut futures = vec![];

    for _ in 0..prompts_per_plant {
        for plant_name in plant_names.iter() {
            futures.push(fetch_conditions(&api_key, plant_name));
        }
    }

    let all_results = join_all(futures).await;
    let mut passing_results: Vec<(String, Conditions)> = all_results
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

    writeln!(out_file, r#""name","shade","moisture""#).unwrap();
    for (name, conditions) in passing_results {
        println!(
            "{} shade: {:?} moisture: {:?}",
            name, conditions.shade, conditions.moisture
        );
        writeln!(
            out_file,
            r#""{}","{:?}","{:?}""#,
            name, conditions.shade, conditions.moisture
        )
        .unwrap();
    }
}

async fn fetch_conditions(api_key: &str, plant_name: &str) -> anyhow::Result<(String, Conditions)> {
    ai::fetch_conditions(api_key, plant_name)
        .await
        .map(|conditions| (plant_name.to_string(), conditions))
}
