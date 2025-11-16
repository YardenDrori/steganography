-- Remove revoked_at column since we now hard delete tokens
ALTER TABLE refresh_tokens DROP COLUMN revoked_at;
