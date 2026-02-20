-- Friends.

CREATE TABLE IF NOT EXISTS friend_relations
(
    from_id UUID NOT NULL,
    to_id   UUID NOT NULL,
    CONSTRAINT pk_friend_relations PRIMARY KEY (from_id, to_id),
    CONSTRAINT fk_friend_relations__friends__from FOREIGN KEY (from_id) REFERENCES users (id),
    CONSTRAINT fk_friend_relations__friends__to FOREIGN KEY (to_id) REFERENCES users (id)
);
