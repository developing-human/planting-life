--liquibase formatted sql

--changeset doug:1
CREATE TABLE zipcodes (
  zipcode INTEGER PRIMARY KEY,
  latitude DECIMAL(7, 4),
  longitude DECIMAL(7, 4)
);
