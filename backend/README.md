# Planting Life Backend

The backend implements APIs to search for native plants, plan gardens, and find local nurseries. It's implemented in Rust and uses a MariaDB database.

# Setup
Pre-requisites: Rust, docker, liquibase.

Create a local copy of the database with `create_dev_db.sh`. This starts a MariaDB database via docker and populates the schema.

Presently this leaves you with the correct schema but an otherwise empty database. I'm planning to add this shortly, but if you want to work on the project please reach out (developinghuman at protonmail dot com) and I'll get this sorted.
# Usage
Run with: `cargo run`
Test with: `cargo test`
Run migrations with: from `db/`, run `liquibase update`

