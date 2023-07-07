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
