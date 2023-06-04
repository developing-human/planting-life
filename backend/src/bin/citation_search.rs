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

    let citations = citations::find(scientific_name).await;

    println!("{citations:#?}");
}
