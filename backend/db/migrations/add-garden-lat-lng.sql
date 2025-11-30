--liquibase formatted sql

--changeset doug:1
ALTER TABLE gardens
ADD COLUMN latitude DECIMAL(16, 14) NULL,
ADD COLUMN longitude DECIMAL(16, 14) NULL;

