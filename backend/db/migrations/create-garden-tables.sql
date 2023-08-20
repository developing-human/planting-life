--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS gardens (
  id INT PRIMARY KEY AUTO_INCREMENT,
  read_id VARCHAR(25),
  write_id VARCHAR(50),
  name VARCHAR(255),
  shade ENUM('None', 'Some', 'Lots'),
  moisture ENUM('None', 'Some', 'Lots'),
  zipcode INT,

  UNIQUE (read_id),
  UNIQUE (write_id)
);

--changeset doug:2
CREATE TABLE IF NOT EXISTS gardens_plants (
  garden_id INT,
  plant_id INT,
  ordering INT,

  PRIMARY KEY (garden_id, plant_id),
  CONSTRAINT FK_GardenPlantsGarden FOREIGN KEY (garden_id) REFERENCES gardens(id),
  CONSTRAINT FK_GardenPlantsPlant FOREIGN KEY (plant_id) REFERENCES plants(id)
);
