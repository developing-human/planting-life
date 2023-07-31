use planting_life::database::Database;
use planting_life::domain::*;
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let db_url = env::var("PLANTING_LIFE_DB_URL").expect("Must define $PLANTING_LIFE_DB_URL");
    let db = Database::new(&db_url);

    //let fetched = db.get_plant_by_scientific_name("Asclepias incarnata").await;
    //println!("fetched: {fetched:#?}");

    for input_zip in [
        "43081", "43083", "", "00123", "4308", "90210", "asdf", "asdfg",
    ] {
        println!("checking {input_zip}");
        match db.lookup_closest_valid_zip(input_zip).await {
            Ok(zip) => println!("closest valid zip: {zip}"),
            Err(e) => println!("error: {e}"),
        }
        println!();
    }

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Expected exactly two arguments, had: {args:?}");
        std::process::exit(1);
    }

    let scientific_name = &args[1];
    let common_name = &args[2];

    let mut plant = Plant::new(scientific_name, common_name);
    plant.moistures = vec![Moisture::Some, Moisture::Lots];
    plant.shades = vec![Shade::None, Shade::Some];

    let saved_plant = db.save_plant(&plant).await;
    println!("saved: {saved_plant:#?}");

    //let fetched_by_id = db.get_plant_by_id(saved_plant.id.unwrap()).await;
    //println!("by id: {fetched_by_id:#?}");

    let fetched_by_name = db.get_plant_by_scientific_name(scientific_name).await;
    println!("by name: {fetched_by_name:#?}");

    /*
    let first = Plant {
        scientific: "first".to_string(),
        common: common_name.to_string(),
        bloom: None,
        image: None,
        id: None,
    };
    let second = Plant {
        scientific: "second".to_string(),
        common: common_name.to_string(),
        bloom: None,
        image: None,
        id: None,
    };
    let third = Plant {
        scientific: "third".to_string(),
        common: common_name.to_string(),
        bloom: None,
        image: None,
        id: None,
    };
    */

    //let first = db.save_plant(&first).await;
    //let all_plants = vec![first, second.clone(), third.clone()];
    //let plants_to_save = vec![second, third];
    //    db.save_query_results(
    //        "43081",
    //        &Moisture::Lots,
    //        &Shade::Some,
    //        &all_plants,
    //        &plants_to_save,
    //    )
    //    .await;
}
