-- Table for raw users.

CREATE TABLE users
(
    id            UUID,
    password_hash TEXT NOT NULL,
    first_name    TEXT NOT NULL,
    second_name   TEXT NOT NULL,
    birthdate     DATE NOT NULL,
    biography     TEXT NOT NULL,
    city          TEXT NOT NULL,
    CONSTRAINT pk_users PRIMARY KEY (id)
);
