use futures::future::join_all;
use planting_life::ai::{openai::OpenAI, Ai, RealAi};
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
    //let prompts_per_plant = 1;
    //let plant_names = vec!["marigolds"];

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut out_file = File::create(format!("temp/prompt-eval-{}.csv", timestamp)).unwrap();
    let mut futures = vec![];

    for _ in 0..prompts_per_plant {
        for plant_name in plant_names.iter() {
            futures.push(fetch_bloom(plant_name));
        }
    }

    let all_results = join_all(futures).await;
    let mut passing_results: Vec<(String, String)> = all_results
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

    writeln!(out_file, r#""name","bloom""#).unwrap();
    for (name, bloom) in passing_results {
        println!("{} {}", name, bloom);
        writeln!(out_file, r#""{}","{}""#, name, bloom).unwrap();
    }

    // Write prompt at end to tie results to prompt
    writeln!(out_file).unwrap();
    writeln!(out_file).unwrap();
    writeln!(out_file).unwrap();
}

async fn fetch_bloom(plant_name: &str) -> anyhow::Result<(String, String)> {
    ai().fetch_bloom(plant_name)
        .await
        .map(|bloom| (plant_name.to_string(), bloom))
}

fn ai() -> Box<dyn Ai> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    Box::new(RealAi {
        open_ai: OpenAI::new(api_key),
    })
}
