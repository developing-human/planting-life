--liquibase formatted sql

--changeset doug:1
CREATE TABLE queries (
  id INT PRIMARY KEY,  
  region_id INT,
  shade ENUM('None', 'Some', 'Lots'),
  moisture ENUM('None', 'Some', 'Lots'),

  FOREIGN KEY(region_id) REFERENCES regions(id)
);

CREATE TABLE queries_plants (
  query_id INT,
  plant_id INT,

  PRIMARY KEY (query_id, plant_id),
  FOREIGN KEY(query_id) REFERENCES queries(id),
  FOREIGN KEY(plant_id) REFERENCES plants(id)
);

--changeset doug:2
ALTER TABLE queries_plants DROP FOREIGN KEY queries_plants_ibfk_1;
--ALTER TABLE queries_plants DROP INDEX queries_plants_ibfk_1;

ALTER TABLE queries MODIFY id INT NOT NULL AUTO_INCREMENT;

ALTER TABLE queries_plants
ADD CONSTRAINT queries_plants_ibfk_1
FOREIGN KEY(query_id) REFERENCES queries(id);


