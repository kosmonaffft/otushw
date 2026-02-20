-- Friends.

CREATE TABLE IF NOT EXISTS posts
(
    id      UUID      NOT NULL,
    user_id UUID      NOT NULL,
    ts      TIMESTAMP NOT NULL,
    content TEXT      NOT NULL,
    CONSTRAINT pk_posts PRIMARY KEY (id),
    CONSTRAINT fk_posts__users FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE INDEX IF NOT EXISTS idx_posts__ts
    ON posts USING btree (ts DESC);
