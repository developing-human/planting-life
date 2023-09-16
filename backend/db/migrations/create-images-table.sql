--liquibase formatted sql

--changeset doug:2
ALTER TABLE plants DROP CONSTRAINT FK_PlantsImage;
ALTER TABLE plants DROP COLUMN image_id;
DROP TABLE images;

--changeset doug:3
CREATE TABLE IF NOT EXISTS images (
  id INT PRIMARY KEY AUTO_INCREMENT,
  plant_id INT, 
  title VARCHAR(200),
  card_url VARCHAR(100),
  original_url VARCHAR(100),
  author VARCHAR(100),
  license VARCHAR(50),
  -- not storing license url, will calculate it

 CONSTRAINT FK_ImagesPlant FOREIGN KEY (plant_id) REFERENCES plants(id)
);
