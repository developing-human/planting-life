use csv::Reader;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, serde::Deserialize)]
struct Record {
    zip: String,
    lat: f64,
    lng: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    //TODO: If ever updating the table of zipcodes, consider rewriting this
    //      to either use batches or use a "loadData" changeset.  This migration
    //      took almost an hour to execute over an ssh tunnel with prod.

    let in_file = File::open("resources/zipcodes.csv")?;
    let mut out_file = File::create("db/migrations/populate-zipcodes.sql")?;
    let mut reader = Reader::from_reader(in_file);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    writeln!(out_file, "--liquibase formatted sql")?;
    writeln!(out_file)?;
    writeln!(out_file, "--changeset script:{timestamp}")?;
    writeln!(out_file, "DELETE FROM zipcodes;")?;
    for result in reader.deserialize() {
        let record: Record = result?;
        writeln!(
            out_file,
            "INSERT INTO zipcodes (zipcode, latitude, longitude) VALUES ('{}', {}, {});",
            record.zip, record.lat, record.lng
        )?;
    }

    Ok(())
}
