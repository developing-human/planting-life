--liquibase formatted sql

--changeset doug:3
CREATE TABLE IF NOT EXISTS plants (
  id INT PRIMARY KEY AUTO_INCREMENT,
  scientific_name VARCHAR(100),
  common_name VARCHAR(100),
  bloom VARCHAR(100),

  pollinator_rating INT,
  pollinator_reason VARCHAR(400),
  bird_rating INT,
  bird_reason VARCHAR(400),

  usda_source VARCHAR(100),
  wiki_source VARCHAR(100),

  -- optional, may not find image
  image_id INT,

  CONSTRAINT FK_PlantsImage FOREIGN KEY (image_id)  REFERENCES images(id)
);
