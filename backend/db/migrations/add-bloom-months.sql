--liquibase formatted sql

--changeset doug:1
ALTER TABLE plants
ADD COLUMN bloom_months SET('Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec');
