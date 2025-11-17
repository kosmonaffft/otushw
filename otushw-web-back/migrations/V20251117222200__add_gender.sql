-- Table for raw users.

ALTER TABLE users
    ADD COLUMN is_male BOOLEAN NOT NULL DEFAULT TRUE;
