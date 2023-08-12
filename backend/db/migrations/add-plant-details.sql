--liquibase formatted sql

--changeset doug:5
ALTER TABLE plants
ADD COLUMN height VARCHAR(50),
ADD COLUMN spread VARCHAR(50);

--changeset doug:6
ALTER TABLE plants
ADD COLUMN spread_rating INT,
ADD COLUMN deer_resistance_rating INT;
