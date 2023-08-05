use std::env;

use planting_life::citations::{Citations, RealCitations};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Expected exactly one argument, had: {args:?}");
        std::process::exit(1);
    }

    let scientific_name = &args[1];

    let citations = RealCitations {};
    let usda_source = citations.find_usda(scientific_name);
    let wiki_source = citations.find_wikipedia(scientific_name).await;

    println!("{usda_source:?}");
    println!("{wiki_source:?}");
}
