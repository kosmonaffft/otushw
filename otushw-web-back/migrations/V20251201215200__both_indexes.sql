-- Indexes for users.

CREATE INDEX idx_users__first_second_name ON users USING btree (first_name, second_name);
