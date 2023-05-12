use std::env;

fn main() {
    let api_key = env::var("OPENAI_API_KEY").expect("Must define $OPENAI_API_KEY");
    let entries = native_plants::fetch_entries(&api_key, "43081", "partial shade", "wet soil");

    for (index, entry) in entries.iter().enumerate() {
        println!("{} ({})", entry.common, entry.scientific);
        println!("{}", entry.description);

        if index != entries.len() - 1 {
            println!();
        }
    }
}
