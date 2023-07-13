--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS zipcodes (
  zipcode INTEGER PRIMARY KEY,
  latitude DECIMAL(7, 4),
  longitude DECIMAL(7, 4),
  region_id INTEGER,

  CONSTRAINT FK_ZipRegion FOREIGN KEY (region_id) REFERENCES regions(id)
);
