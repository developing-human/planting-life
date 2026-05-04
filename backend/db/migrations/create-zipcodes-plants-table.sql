--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS zipcodes_plants (
    zipcode INT,
    plant_id INT,

    PRIMARY KEY (zipcode, plant_id),

    CONSTRAINT ZipcodesPlantsZipcode FOREIGN KEY(zipcode) REFERENCES zipcodes(zipcode),
    CONSTRAINT ZipcodesPlantsPlant FOREIGN KEY(plant_id) REFERENCES plants(id)
);
