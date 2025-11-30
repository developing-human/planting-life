
--liquibase formatted sql

--changeset doug:1
CREATE TABLE IF NOT EXISTS request_counts (
  uri VARCHAR(100) NOT NULL,
  date DATE NOT NULL DEFAULT now(),
  count INT UNSIGNED NOT NULL DEFAULT 1,

  PRIMARY KEY(uri, date)
);

-- use like:
-- INSERT INTO request_counts (uri, count) 
-- VALUES ('/maps/api-key', 1) 
-- ON DUPLICATE KEY UPDATE 
--   count = count + 1;
