-- Indexes for users.

CREATE INDEX idx_users__first_name ON users USING btree (first_name text_pattern_ops);
CREATE INDEX idx_users__second_name ON users USING btree (second_name text_pattern_ops);
CREATE INDEX idx_users__first_second_name ON users USING btree (first_name, second_name text_pattern_ops);

ANALYZE
    users;
