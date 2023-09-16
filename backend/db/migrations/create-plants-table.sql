--liquibase formatted sql

--changeset doug:9
CREATE TABLE IF NOT EXISTS plants (
  id INT PRIMARY KEY AUTO_INCREMENT,
  scientific_name VARCHAR(100),
  common_name VARCHAR(100),
  bloom VARCHAR(100),

  pollinator_rating INT,
  bird_rating INT,

  usda_source VARCHAR(100),
  wiki_source VARCHAR(100),

  CONSTRAINT UC_Plants UNIQUE (scientific_name),
  FULLTEXT (scientific_name, common_name)
);

--changeset doug:8
ALTER TABLE plants ADD CONSTRAINT UC_Plants UNIQUE (scientific_name);