--liquibase formatted sql

--changeset doug:1
CREATE TABLE plants (
  id INT PRIMARY KEY,
  scientific_name VARCHAR(100),
  common_name VARCHAR(100),
  description VARCHAR(400),

  -- optional, may not find image
  image_id INT,
  FOREIGN KEY(image_id) REFERENCES images(id)
);

--changeset doug:2
ALTER TABLE queries_plants DROP FOREIGN KEY queries_plants_ibfk_2;
ALTER TABLE queries_plants DROP INDEX queries_plants_ibfk_2;

ALTER TABLE plants MODIFY id INT NOT NULL AUTO_INCREMENT;

ALTER TABLE queries_plants
ADD CONSTRAINT queries_plants_ibfk_2
FOREIGN KEY(plant_id) REFERENCES plants(id);

--changeset doug:3
ALTER TABLE plants ADD COLUMN bloom VARCHAR(100);
