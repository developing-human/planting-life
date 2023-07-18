use planting_life::ai;
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Expected exactly one argument, had: {args:?}");
        std::process::exit(1);
    }

    let scientific_name = &args[1];

    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let rating = ai::fetch_pollinator_rating(&api_key, scientific_name).await;

    println!("{rating:#?}");
}
