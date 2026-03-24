-- Dialogues.

CREATE TABLE IF NOT EXISTS dialogues
(
    id       UUID      NOT NULL,
    from_id  UUID      NOT NULL,
    to_id    UUID      NOT NULL,
    distr_id UUID      NOT NULL,
    ts       TIMESTAMP NOT NULL,
    content  TEXT      NOT NULL,
    CONSTRAINT pk_dialogues PRIMARY KEY (id, distr_id)
);

SELECT create_distributed_table('dialogues', 'distr_id');
