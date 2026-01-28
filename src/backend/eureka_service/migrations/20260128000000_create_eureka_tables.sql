CREATE TABLE shared_config (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    jwt_private_key TEXT NOT NULL,
    jwt_public_key TEXT NOT NULL,
    internal_api_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE service_registry (
    id BIGSERIAL PRIMARY KEY,
    service_name VARCHAR(100) NOT NULL UNIQUE,
    service_url VARCHAR(500) NOT NULL,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_service_registry_name ON service_registry(service_name);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_shared_config_updated_at
    BEFORE UPDATE ON shared_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
