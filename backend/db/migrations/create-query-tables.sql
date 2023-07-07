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
