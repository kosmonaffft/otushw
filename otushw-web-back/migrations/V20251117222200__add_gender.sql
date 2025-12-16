-- add is_male column.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS is_male BOOLEAN NOT NULL DEFAULT TRUE;
