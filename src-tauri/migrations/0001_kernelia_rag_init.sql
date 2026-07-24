CREATE TABLE IF NOT EXISTS knowledge_document (
    id TEXT PRIMARY KEY,
    specialty_id TEXT NOT NULL,
    doc_type TEXT NOT NULL,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    summary TEXT NOT NULL DEFAULT '',
    body_markdown TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_path TEXT NOT NULL DEFAULT '',
    version TEXT NOT NULL DEFAULT '1',
    status TEXT NOT NULL DEFAULT 'draft',
    content_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS knowledge_chunk (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    specialty_id TEXT NOT NULL,
    entity_key TEXT,
    title_anchor TEXT NOT NULL DEFAULT '',
    lexical_weight REAL NOT NULL DEFAULT 1.0,
    semantic_weight REAL NOT NULL DEFAULT 1.0,
    risk_level_hint TEXT NOT NULL DEFAULT 'r0',
    FOREIGN KEY(document_id) REFERENCES knowledge_document(id)
);

CREATE TABLE IF NOT EXISTS knowledge_chunk_embedding (
    id TEXT PRIMARY KEY,
    chunk_id TEXT NOT NULL,
    embedding_provider TEXT NOT NULL,
    embedding_model TEXT NOT NULL,
    embedding_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(chunk_id) REFERENCES knowledge_chunk(id)
);

CREATE TABLE IF NOT EXISTS knowledge_relation (
    id TEXT PRIMARY KEY,
    from_document_id TEXT NOT NULL,
    to_document_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    FOREIGN KEY(from_document_id) REFERENCES knowledge_document(id),
    FOREIGN KEY(to_document_id) REFERENCES knowledge_document(id)
);

CREATE TABLE IF NOT EXISTS windows_command (
    id TEXT PRIMARY KEY,
    canonical_name TEXT NOT NULL UNIQUE,
    shell_type TEXT NOT NULL,
    command_template TEXT NOT NULL,
    description TEXT NOT NULL,
    specialty_id TEXT NOT NULL,
    area_key TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    is_read_only INTEGER NOT NULL DEFAULT 1,
    supports_dry_run INTEGER NOT NULL DEFAULT 0,
    supports_rollback INTEGER NOT NULL DEFAULT 0,
    requires_admin INTEGER NOT NULL DEFAULT 0,
    requires_owner INTEGER NOT NULL DEFAULT 0,
    requires_megaboss INTEGER NOT NULL DEFAULT 0,
    expected_output_kind TEXT NOT NULL DEFAULT 'text',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS windows_command_alias (
    id TEXT PRIMARY KEY,
    command_id TEXT NOT NULL,
    alias_text TEXT NOT NULL,
    alias_kind TEXT NOT NULL,
    FOREIGN KEY(command_id) REFERENCES windows_command(id)
);

CREATE TABLE IF NOT EXISTS tool_capability (
    id TEXT PRIMARY KEY,
    tool_name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    specialty_id TEXT NOT NULL,
    description TEXT NOT NULL,
    input_schema_json TEXT NOT NULL,
    output_schema_json TEXT NOT NULL,
    min_role TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    mutability_type TEXT NOT NULL,
    verification_required INTEGER NOT NULL DEFAULT 0,
    snapshot_required INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS tool_command_binding (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    command_id TEXT NOT NULL,
    binding_mode TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(tool_id) REFERENCES tool_capability(id),
    FOREIGN KEY(command_id) REFERENCES windows_command(id)
);

CREATE TABLE IF NOT EXISTS tool_precondition (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    rule_code TEXT NOT NULL,
    rule_description TEXT NOT NULL,
    FOREIGN KEY(tool_id) REFERENCES tool_capability(id)
);

CREATE TABLE IF NOT EXISTS tool_postcondition (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    verify_tool_name TEXT NOT NULL,
    verify_args_json TEXT NOT NULL,
    expected_condition TEXT NOT NULL,
    FOREIGN KEY(tool_id) REFERENCES tool_capability(id)
);

CREATE TABLE IF NOT EXISTS tool_guardrail (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    guardrail_code TEXT NOT NULL,
    guardrail_description TEXT NOT NULL,
    FOREIGN KEY(tool_id) REFERENCES tool_capability(id)
);

CREATE TABLE IF NOT EXISTS tool_evidence_rule (
    id TEXT PRIMARY KEY,
    tool_id TEXT NOT NULL,
    evidence_kind TEXT NOT NULL,
    evidence_description TEXT NOT NULL,
    required_before INTEGER NOT NULL DEFAULT 0,
    required_after INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(tool_id) REFERENCES tool_capability(id)
);

CREATE TABLE IF NOT EXISTS query_category (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS domain_specialty (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    agent_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS symptom_taxonomy (
    id TEXT PRIMARY KEY,
    symptom_key TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    specialty_id TEXT NOT NULL,
    severity_default TEXT NOT NULL,
    common_causes_json TEXT NOT NULL,
    recommended_checks_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS intent_to_specialty_rule (
    id TEXT PRIMARY KEY,
    match_type TEXT NOT NULL,
    match_value TEXT NOT NULL,
    specialty_id TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0,
    requires_exact INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS decision_policy (
    id TEXT PRIMARY KEY,
    query_category_id TEXT NOT NULL,
    specialty_id TEXT NOT NULL,
    confidence_min REAL NOT NULL,
    risk_max_auto TEXT NOT NULL,
    decision_mode TEXT NOT NULL,
    requires_clarification INTEGER NOT NULL DEFAULT 0,
    requires_live_state INTEGER NOT NULL DEFAULT 0,
    requires_snapshot INTEGER NOT NULL DEFAULT 0,
    requires_human INTEGER NOT NULL DEFAULT 0,
    response_style TEXT NOT NULL DEFAULT 'technical',
    FOREIGN KEY(query_category_id) REFERENCES query_category(id)
);

CREATE TABLE IF NOT EXISTS confidence_policy (
    id TEXT PRIMARY KEY,
    query_category_id TEXT NOT NULL,
    specialty_id TEXT NOT NULL,
    vector_score_weight REAL NOT NULL DEFAULT 0.0,
    lexical_score_weight REAL NOT NULL DEFAULT 1.0,
    exact_match_bonus REAL NOT NULL DEFAULT 0.0,
    specialty_match_bonus REAL NOT NULL DEFAULT 0.0,
    live_state_bonus REAL NOT NULL DEFAULT 0.0,
    tool_verifiability_bonus REAL NOT NULL DEFAULT 0.0,
    ambiguity_penalty REAL NOT NULL DEFAULT 0.0,
    short_query_penalty REAL NOT NULL DEFAULT 0.0,
    conflict_penalty REAL NOT NULL DEFAULT 0.0,
    high_threshold REAL NOT NULL,
    medium_threshold REAL NOT NULL,
    FOREIGN KEY(query_category_id) REFERENCES query_category(id)
);

CREATE TABLE IF NOT EXISTS risk_policy (
    id TEXT PRIMARY KEY,
    tool_name TEXT NOT NULL UNIQUE,
    risk_level TEXT NOT NULL,
    min_role TEXT NOT NULL,
    requires_snapshot INTEGER NOT NULL DEFAULT 0,
    requires_post_verify INTEGER NOT NULL DEFAULT 0,
    requires_megaboss INTEGER NOT NULL DEFAULT 0,
    allow_auto_execute INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS execution_policy (
    id TEXT PRIMARY KEY,
    specialty_id TEXT NOT NULL,
    query_category_id TEXT NOT NULL,
    confidence_min_for_explain REAL NOT NULL,
    confidence_min_for_simulate REAL NOT NULL,
    confidence_min_for_execute REAL NOT NULL,
    max_risk_auto TEXT NOT NULL,
    FOREIGN KEY(query_category_id) REFERENCES query_category(id)
);

CREATE TABLE IF NOT EXISTS escalation_policy (
    id TEXT PRIMARY KEY,
    specialty_id TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    escalation_target TEXT NOT NULL,
    reason_template TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS clarification_template (
    id TEXT PRIMARY KEY,
    specialty_id TEXT NOT NULL,
    query_category_id TEXT NOT NULL,
    template_text TEXT NOT NULL,
    target_slot TEXT NOT NULL,
    FOREIGN KEY(query_category_id) REFERENCES query_category(id)
);

CREATE TABLE IF NOT EXISTS conversation_session (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    latest_specialty TEXT NOT NULL DEFAULT 'unknown',
    latest_risk_level TEXT NOT NULL DEFAULT 'r0'
);

CREATE TABLE IF NOT EXISTS conversation_message (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(session_id) REFERENCES conversation_session(id)
);

CREATE TABLE IF NOT EXISTS memory_snapshot (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    latest_intent TEXT NOT NULL,
    latest_specialty TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    confidence REAL NOT NULL,
    decision_mode TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(session_id) REFERENCES conversation_session(id)
);

CREATE TABLE IF NOT EXISTS memory_fact (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    fact_type TEXT NOT NULL,
    fact_key TEXT NOT NULL,
    fact_value TEXT NOT NULL,
    confidence REAL NOT NULL,
    FOREIGN KEY(snapshot_id) REFERENCES memory_snapshot(id)
);

CREATE TABLE IF NOT EXISTS memory_tag (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    FOREIGN KEY(snapshot_id) REFERENCES memory_snapshot(id)
);

CREATE TABLE IF NOT EXISTS memory_component_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    component_name TEXT NOT NULL,
    state_value TEXT NOT NULL,
    FOREIGN KEY(snapshot_id) REFERENCES memory_snapshot(id)
);

CREATE TABLE IF NOT EXISTS memory_action_history (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    action_mode TEXT NOT NULL,
    result_status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(session_id) REFERENCES conversation_session(id)
);

CREATE TABLE IF NOT EXISTS memory_open_hypothesis (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    hypothesis_label TEXT NOT NULL,
    confidence REAL NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL,
    FOREIGN KEY(session_id) REFERENCES conversation_session(id)
);

CREATE TABLE IF NOT EXISTS trace_request (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL UNIQUE,
    session_id TEXT,
    user_message TEXT NOT NULL,
    normalized_message TEXT NOT NULL,
    query_category TEXT NOT NULL,
    specialty_detected TEXT NOT NULL,
    requires_live_state INTEGER NOT NULL DEFAULT 0,
    latency_ms_total INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trace_retrieval_hit (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    title TEXT NOT NULL,
    score_vector REAL NOT NULL DEFAULT 0.0,
    score_lexical REAL NOT NULL DEFAULT 0.0,
    score_final REAL NOT NULL DEFAULT 0.0,
    entity_key TEXT,
    used_in_context INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS trace_confidence (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    confidence_level TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    should_use_context INTEGER NOT NULL DEFAULT 0,
    should_ask_clarifying_question INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS trace_decision (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    decision_mode TEXT NOT NULL,
    reason_text TEXT NOT NULL,
    used_context INTEGER NOT NULL DEFAULT 0,
    used_live_state INTEGER NOT NULL DEFAULT 0,
    used_tools INTEGER NOT NULL DEFAULT 0,
    escalated INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS trace_tool_call (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    args_json TEXT NOT NULL,
    result_status TEXT NOT NULL,
    verification_status TEXT NOT NULL DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS trace_verification (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    verification_tool_name TEXT NOT NULL,
    verification_result TEXT NOT NULL,
    verified INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS trace_error (
    id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    stage_name TEXT NOT NULL,
    error_code TEXT NOT NULL,
    error_message TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS endpoint_snapshot (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL,
    hostname TEXT NOT NULL DEFAULT '',
    os_name TEXT NOT NULL DEFAULT '',
    raw_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS endpoint_cpu_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    usage_percent REAL NOT NULL DEFAULT 0.0,
    model_name TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE TABLE IF NOT EXISTS endpoint_memory_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    used_bytes INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE TABLE IF NOT EXISTS endpoint_disk_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    mount_point TEXT NOT NULL,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    used_bytes INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE TABLE IF NOT EXISTS endpoint_network_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    adapter_name TEXT NOT NULL,
    ip_address TEXT NOT NULL DEFAULT '',
    gateway TEXT NOT NULL DEFAULT '',
    dns_servers TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE TABLE IF NOT EXISTS endpoint_process_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    pid INTEGER NOT NULL,
    process_name TEXT NOT NULL,
    cpu_percent REAL NOT NULL DEFAULT 0.0,
    memory_bytes INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE TABLE IF NOT EXISTS endpoint_service_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    service_name TEXT NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE TABLE IF NOT EXISTS endpoint_driver_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    device_name TEXT NOT NULL,
    driver_status TEXT NOT NULL,
    error_code TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE TABLE IF NOT EXISTS endpoint_security_state (
    id TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    firewall_status TEXT NOT NULL DEFAULT '',
    defender_status TEXT NOT NULL DEFAULT '',
    open_ports_summary TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(snapshot_id) REFERENCES endpoint_snapshot(id)
);

CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_chunk_fts USING fts5(
    chunk_text,
    title_anchor,
    entity_key,
    content='knowledge_chunk',
    content_rowid='rowid'
);

CREATE VIRTUAL TABLE IF NOT EXISTS windows_command_alias_fts USING fts5(
    alias_text,
    alias_kind,
    content='windows_command_alias',
    content_rowid='rowid'
);

CREATE INDEX IF NOT EXISTS idx_knowledge_chunk_specialty ON knowledge_chunk(specialty_id);
CREATE INDEX IF NOT EXISTS idx_windows_command_specialty ON windows_command(specialty_id);
CREATE INDEX IF NOT EXISTS idx_tool_capability_specialty ON tool_capability(specialty_id);
CREATE INDEX IF NOT EXISTS idx_trace_request_session ON trace_request(session_id);
CREATE INDEX IF NOT EXISTS idx_memory_snapshot_session ON memory_snapshot(session_id);
