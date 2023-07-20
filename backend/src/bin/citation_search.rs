use planting_life::citations;
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Expected exactly one argument, had: {args:?}");
        std::process::exit(1);
    }

    let scientific_name = &args[1];

    let usda_source = citations::usda::find(scientific_name);
    let wiki_source = citations::wikipedia::find(scientific_name).await;

    println!("{usda_source:?}");
    println!("{wiki_source:?}");
}
