--liquibase formatted sql

--changeset doug:1
ALTER TABLE plants
ADD COLUMN pollinator_rating INT,
ADD COLUMN pollinator_reason VARCHAR(400),
ADD COLUMN bird_rating INT,
ADD COLUMN bird_reason VARCHAR(400),
ADD COLUMN animal_rating INT,
ADD COLUMN animal_reason VARCHAR(400),
DROP COLUMN description;

--changeset doug:3
ALTER TABLE plants
ADD COLUMN usda_source VARCHAR(100),
ADD COLUMN wiki_source VARCHAR(100);

--changeset doug:4
ALTER TABLE plants
MODIFY COLUMN pollinator_reason VARCHAR(600);

ALTER TABLE plants
MODIFY COLUMN bird_reason VARCHAR(600);

ALTER TABLE plants
MODIFY COLUMN animal_reason VARCHAR(600);

--changeset doug:5
ALTER TABLE plants
ADD COLUMN height VARCHAR(50),
ADD COLUMN spread VARCHAR(50);

--changeset doug:6
ALTER TABLE plants
ADD COLUMN spread_rating INT,
ADD COLUMN deer_resistance_rating INT;

--changeset doug:7
ALTER TABLE plants
DROP COLUMN animal_rating,
DROP COLUMN animal_reason;

--changeset doug:8
ALTER TABLE plants
DROP COLUMN pollinator_reason,
DROP COLUMN bird_reason;
