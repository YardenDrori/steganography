-- Add migration script here
CREATE TABLE user_roles (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role)
);

-- Index for querying roles by user
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);

-- Insert default 'user' role for all existing users
INSERT INTO user_roles (user_id, role)
SELECT id, 'user' FROM users;
