--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS nurseries (
  id INT PRIMARY KEY AUTO_INCREMENT,
  name VARCHAR(255),
  url VARCHAR(255),
  address VARCHAR(255),
  city VARCHAR(50),
  state VARCHAR(2),
  zipcode INT,
  latitude DECIMAL(7,4),
  longitude DECIMAL(7,4)
);


--changeset doug:2
CREATE TABLE zipcodes_nurseries (
 zipcode INT,
 nursery_id INT,
 miles DECIMAL(3),
 PRIMARY KEY (zipcode, nursery_id),
 CONSTRAINT FK_ZipNurseriesZip FOREIGN KEY (zipcode) REFERENCES zipcodes(zipcode),
 CONSTRAINT FK_ZipNurseriesNursery FOREIGN KEY (nursery_id) REFERENCES nurseries(id)
);
