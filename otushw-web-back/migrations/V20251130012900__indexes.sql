-- Indexes for users.

CREATE INDEX idx_users__first_name ON users USING btree (first_name);
CREATE INDEX idx_users__second_name ON users USING btree (second_name);
