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
