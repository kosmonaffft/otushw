-- Table for raw users.

CREATE TABLE users
(
    id            BIGSERIAL,
    login         TEXT    NOT NULL,
    password_hash TEXT    NOT NULL,
    name          TEXT    NOT NULL,
    family_name   TEXT    NOT NULL,
    birthdate     DATE    NOT NULL,
    is_male       BOOLEAN NOT NULL,
    interests     TEXT    NOT NULL,
    city          TEXT    NOT NULL,
    CONSTRAINT pk_users PRIMARY KEY (id),
    CONSTRAINT uq_users__login UNIQUE (login)
);
