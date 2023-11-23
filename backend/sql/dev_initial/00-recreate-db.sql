-- NOTE: This is for early stage fast development. 
-- Another important note is we're not using the PG defaults
-- for the username and database name, since it's harder to
-- change later on. This way we're not directly creating
-- under PG defaults.

-- DEV ONLY - Brute Force Drop DB (for local dev and unit testing)
SELECT pg_terminate_backend(pid)
  FROM pg_stat_activity
  WHERE usename = 'app_user' OR datname = 'app_db';
DROP DATABASE IF EXISTS app_db;
DROP USER IF EXISTS app_user;

-- DEV ONLY - Dev only password (for local dev and unit testing)
CREATE user app_user PASSWORD 'dev_only_pwd';
CREATE DATABASE app_db owner app_user ENCODING = 'utf-8';
