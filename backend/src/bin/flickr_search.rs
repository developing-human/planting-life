use planting_life::flickr::{Flickr, RealFlickr};
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let flickr_api_key = env::var("FLICKR_API_KEY").expect("Must define $FLICKR_API_KEY");
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Expected exactly two arguments, had: {args:?}");
        std::process::exit(1);
    }

    let scientific_name = &args[1];
    let common_name = &args[2];

    let flickr = RealFlickr::new(flickr_api_key);
    let image = flickr.get_image(scientific_name, common_name).await;

    println!("{image:#?}");
}
