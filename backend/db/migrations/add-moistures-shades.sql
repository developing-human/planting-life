--liquibase formatted sql

--changeset doug:1
ALTER TABLE plants
ADD COLUMN moistures SET('None', 'Some', 'Lots'),
ADD COLUMN shades SET('None', 'Some', 'Lots');
