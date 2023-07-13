--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS plants (
  id INT PRIMARY KEY AUTO_INCREMENT,
  scientific_name VARCHAR(100),
  common_name VARCHAR(100),
  description VARCHAR(400),
  bloom VARCHAR(100),

  -- optional, may not find image
  image_id INT,

  CONSTRAINT FK_PlantsImage FOREIGN KEY (image_id)  REFERENCES images(id)
);
