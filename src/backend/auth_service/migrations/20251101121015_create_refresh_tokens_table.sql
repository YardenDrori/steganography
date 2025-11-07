CREATETABLErefresh_tokens(
  id BIGSERIALPRIMARYKEY,
  user_idBIGINTNOTNULLREFERENCESusers(id)
    ON DELETECASCADE,
    token_hashVARCHAR(64)NOTNULLUNIQUE,
    -- SHA-256 hash = 64 hex chars
expires_at TIMESTAMPTZNOTNULL,
    created_at TIMESTAMPTZNOTNULLDEFAULTCURRENT_TIMESTAMP,
    last_used_at TIMESTAMPTZ,
    is_revoked TIMESTAMPTZDEFAULTNULL,
    device_infoTEXT-- Optional: store user agent, device name, etc.

);
CREATEINDEX idx_refresh_tokens_user_id
ON refresh_tokens(user_id);
CREATEINDEX idx_refresh_tokens_token_hash
ON refresh_tokens(token_hash);
CREATEINDEX idx_refresh_tokens_expires_at
ON refresh_tokens(expires_at);
