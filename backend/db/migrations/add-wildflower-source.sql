--liquibase formatted sql

--changeset doug:1
ALTER TABLE plants ADD COLUMN wildflower_source VARCHAR(100);
