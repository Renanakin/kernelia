CREATE TABLE IF NOT EXISTS support_ticket (
    id TEXT PRIMARY KEY,
    ticket_code TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    priority TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    specialty TEXT NOT NULL DEFAULT 'General',
    customer_id TEXT NOT NULL DEFAULT 'local_user',
    telemetry_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_feedback_log (
    id TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    solution_text TEXT NOT NULL,
    satisfied INTEGER NOT NULL,
    source_type TEXT NOT NULL,
    created_at TEXT NOT NULL
);
