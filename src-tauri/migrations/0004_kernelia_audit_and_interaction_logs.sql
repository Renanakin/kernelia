-- Migración 0004: Tabla de Auditoría de Interacciones y Desafíos de Seguridad KernelIA

CREATE TABLE IF NOT EXISTS user_interaction_log (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    user_role TEXT NOT NULL CHECK(user_role IN ('standard_user', 'tech_analyst', 'superadmin')),
    query_text TEXT NOT NULL,
    intent_detected TEXT NOT NULL,
    response_mode TEXT NOT NULL CHECK(response_mode IN ('written_solution', 'auto_exec_request', 'elevation_challenge')),
    action_requested TEXT,
    command_risk_level TEXT CHECK(command_risk_level IN ('R0', 'R1', 'R2', 'R3', 'R4')),
    elevation_required INTEGER NOT NULL DEFAULT 0,
    elevation_status TEXT CHECK(elevation_status IN ('NOT_REQUIRED', 'PASSED', 'DENIED', 'CANCELLED')),
    authenticated_by TEXT,
    execution_result TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_interaction_session ON user_interaction_log(session_id);
CREATE INDEX IF NOT EXISTS idx_interaction_risk ON user_interaction_log(command_risk_level);
CREATE INDEX IF NOT EXISTS idx_interaction_user ON user_interaction_log(user_id);
