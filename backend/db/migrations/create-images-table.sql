--liquibase formatted sql

--changeset doug:1
CREATE TABLE images (
  id INT PRIMARY KEY,
  title VARCHAR(200),
  card_url VARCHAR(100),
  original_url VARCHAR(100),
  author VARCHAR(100),
  license VARCHAR(50)
  -- not storing license url, will calculate it

);

--changeset doug:2
ALTER TABLE plants DROP FOREIGN KEY plants_ibfk_1;
ALTER TABLE plants DROP INDEX plants_ibfk_1;

ALTER TABLE images MODIFY id INT NOT NULL AUTO_INCREMENT;

ALTER TABLE plants
ADD CONSTRAINT plants_ibfk_1
FOREIGN KEY(image_id) REFERENCES images(id);

