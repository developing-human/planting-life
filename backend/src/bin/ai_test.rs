use planting_life::ai::{openai::OpenAI, Ai, RealAi};
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Expected exactly one argument, had: {args:?}");
        std::process::exit(1);
    }

    let scientific_name = &args[1];

    let rating = ai().fetch_pollinator_rating(scientific_name).await;

    println!("{rating:#?}");
}

fn ai() -> Box<dyn Ai> {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");

    Box::new(RealAi {
        open_ai: OpenAI::new(api_key),
    })
}
