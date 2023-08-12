--liquibase formatted sql

--changeset doug:2
CREATE TABLE IF NOT EXISTS queries (
  id INT PRIMARY KEY AUTO_INCREMENT,  
  region_id INT,
  shade ENUM('None', 'Some', 'Lots'),
  moisture ENUM('None', 'Some', 'Lots'),
  count INT,

  CONSTRAINT FK_QueriesRegion FOREIGN KEY(region_id) REFERENCES regions(id),
  CONSTRAINT UC_Queries UNIQUE (region_id, shade, moisture)
);

CREATE TABLE IF NOT EXISTS queries_plants (
  query_id INT,
  plant_id INT,

  PRIMARY KEY (query_id, plant_id),
  CONSTRAINT QueryPlantsQuery FOREIGN KEY(query_id) REFERENCES queries(id),
  CONSTRAINT QueryPlantsPlant FOREIGN KEY(plant_id) REFERENCES plants(id)
);

--changeset doug:4
UPDATE queries SET count = 0;
