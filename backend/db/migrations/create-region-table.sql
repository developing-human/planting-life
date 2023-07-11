--liquibase formatted sql

--changeset doug:1
CREATE TABLE regions (
  id INTEGER PRIMARY KEY,
  name VARCHAR(100),
  latitude DECIMAL(7, 4),
  longitude DECIMAL(7, 4),
  radius_miles INTEGER
);

-- each zipcode references the region it is part of
-- all zip codes have a region, even if its just the one zipcode
ALTER TABLE zipcodes
ADD COLUMN region_id INT;

ALTER TABLE zipcodes
ADD CONSTRAINT FK_REGION
FOREIGN KEY(region_id) REFERENCES regions(id);

--changeset doug:2
ALTER TABLE zipcodes DROP FOREIGN KEY FK_REGION;
ALTER TABLE zipcodes DROP INDEX FK_REGION;
ALTER TABLE queries DROP FOREIGN KEY queries_ibfk_1;
ALTER TABLE queries DROP INDEX queries_ibfk_1;

ALTER TABLE regions MODIFY id INT NOT NULL AUTO_INCREMENT;

ALTER TABLE zipcodes
ADD CONSTRAINT FK_REGION
FOREIGN KEY(region_id) REFERENCES regions(id);

ALTER TABLE queries
ADD CONSTRAINT queries_ibfk_1
FOREIGN KEY(region_id) REFERENCES regions(id);
