--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS regions (
  id INTEGER PRIMARY KEY AUTO_INCREMENT,
  name VARCHAR(100),
  latitude DECIMAL(7, 4),
  longitude DECIMAL(7, 4),
  radius_miles INTEGER
);
