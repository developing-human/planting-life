--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS regions_plants (
    region_id INT,
    plant_id INT,

    PRIMARY KEY (region_id, plant_id),

    CONSTRAINT RegionPlantsRegion FOREIGN KEY(region_id) REFERENCES regions(id),
    CONSTRAINT RegionPlantsPlant FOREIGN KEY(plant_id) REFERENCES plants(id)
);

--changeset doug:2
INSERT INTO regions_plants (region_id, plant_id)
SELECT UNIQUE region_id, plant_id
FROM queries q
INNER JOIN queries_plants qp ON qp.query_id = q.id
WHERE region_id is not null;

