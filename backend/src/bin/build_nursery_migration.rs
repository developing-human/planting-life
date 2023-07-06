use csv::Reader;
use geo::HaversineDistance;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct ZipCode {
    zip: String,
    lat: f64,
    lng: f64,
}

#[derive(Debug, Deserialize)]
struct Nursery {
    id: usize,
    name: String,
    url: String,
    address: String,
    city: String,
    state: String,
    zip: String,
    lat: f64,
    long: f64,
}

fn load_zips() -> Vec<ZipCode> {
    let mut reader = Reader::from_path("resources/zipcodes.csv").unwrap();
    let mut zipcodes = Vec::new();

    for result in reader.deserialize() {
        let record: ZipCode = result.unwrap();
        zipcodes.push(record);
    }

    zipcodes
}

fn load_nurseries() -> Vec<Nursery> {
    let mut reader = Reader::from_path("resources/nurseries.csv").unwrap();
    let mut locations = Vec::new();

    for result in reader.deserialize() {
        let record: Nursery = result.unwrap();
        locations.push(record);
    }

    locations
}

fn miles_between(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let location1 = geo::Point::new(lon1, lat1);
    let location2 = geo::Point::new(lon2, lat2);
    location1.haversine_distance(&location2) / 1609.344 // convert meters to miles
}

fn main() -> Result<(), Box<dyn Error>> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let zips = load_zips();
    let nurseries = load_nurseries();
    let mut out_file = File::create("db/migrations/populate-nurseries.sql")?;

    writeln!(out_file, "--liquibase formatted sql")?;

    writeln!(out_file, "--changeset script:0-{}", timestamp)?;
    writeln!(out_file, "DELETE from zipcodes_nurseries;")?;
    writeln!(out_file, "DELETE from nurseries;")?;

    for nursery in nurseries {
        writeln!(out_file)?;
        writeln!(out_file, "--changeset script:{}-{}", nursery.id, timestamp)?;
        writeln!(
            out_file,
            "INSERT INTO nurseries 
  (id, name, url, address, city, state, zipcode, latitude, longitude) 
VALUES 
  ({}, '{}', {}, '{}', '{}', '{}', {}, {}, {});\n",
            nursery.id,
            nursery.name.replace('\'', "''"),
            if nursery.url.is_empty() {
                "null".to_string()
            } else {
                format!("'{}'", nursery.url)
            },
            nursery.address.replace('\'', "''"),
            nursery.city,
            nursery.state,
            nursery.zip,
            nursery.lat,
            nursery.long
        )?;

        let mut to_write = vec![];
        for zip in zips.iter() {
            let miles = miles_between(zip.lat, zip.lng, nursery.lat, nursery.long);
            if miles <= 75.0 {
                to_write.push((&zip.zip, miles.round() as usize))
            }
        }

        if !to_write.is_empty() {
            writeln!(
                out_file,
                "INSERT INTO zipcodes_nurseries (zipcode, nursery_id, miles) VALUES",
            )?;

            let rows = to_write
                .iter()
                .map(|(zip, miles)| format!("({}, {}, {})", zip, nursery.id, miles))
                .collect::<Vec<_>>()
                .join(",\n  ");

            writeln!(out_file, "  {rows};")?;
        }
    }

    Ok(())
}
