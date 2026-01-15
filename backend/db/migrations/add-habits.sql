--liquibase formatted sql

--changeset doug:2
ALTER TABLE plants
ADD COLUMN habits SET( 'Garden', 'Grass', 'Shrub', 'Tree', 'Vine' );

--changeset doug:3
--renames Garden to FlowerOrHerb
ALTER TABLE plants
CHANGE COLUMN habits habits SET( 'FlowerOrHerb', 'Grass', 'Shrub', 'Tree', 'Vine' );
