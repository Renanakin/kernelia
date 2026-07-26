CREATE TABLE IF NOT EXISTS hitl_checkpoint (
    id TEXT PRIMARY KEY,
    checkpoint_code TEXT NOT NULL UNIQUE,
    session_id TEXT NOT NULL DEFAULT '',
    tool_name TEXT NOT NULL,
    args_json TEXT NOT NULL DEFAULT '{}',
    risk_level TEXT NOT NULL,
    required_role TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    requested_at TEXT NOT NULL,
    resolved_at TEXT,
    resolved_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_hitl_checkpoint_status ON hitl_checkpoint(status);
CREATE INDEX IF NOT EXISTS idx_hitl_checkpoint_session ON hitl_checkpoint(session_id);
