INSERT INTO query_category (id, code, description) VALUES
('qc_specific', 'specific', 'Consulta tecnica concreta con objeto claro'),
('qc_short', 'short', 'Consulta corta que requiere desambiguacion'),
('qc_ambiguous', 'ambiguous', 'Consulta ambigua con multiples interpretaciones'),
('qc_symptom_based', 'symptom_based', 'Consulta basada en sintomas observados'),
('qc_action_request', 'action_request', 'Solicitud explicita de accion'),
('qc_unsafe_request', 'unsafe_request', 'Solicitud potencialmente insegura'),
('qc_out_of_domain', 'out_of_domain', 'Consulta fuera del dominio operativo')
ON CONFLICT(id) DO NOTHING;

INSERT INTO domain_specialty (id, code, name, description, agent_name) VALUES
('sp_system', 'system', 'System', 'Informacion estructural del sistema', 'KernelIA-System-Profiler'),
('sp_telemetry', 'telemetry', 'Telemetry', 'Telemetria y baseline operacional', 'KernelIA-Realtime-Telemetry'),
('sp_network', 'network', 'Network', 'Red, DNS, conectividad y stack de internet', 'KernelIA-Network-Intel'),
('sp_processes', 'processes', 'Processes', 'Analisis y control de procesos', 'KernelIA-Process-Guardian'),
('sp_services', 'services', 'Services', 'Servicios de Windows y continuidad', 'KernelIA-Service-Controller'),
('sp_maintenance', 'maintenance', 'Maintenance', 'Mantenimiento basico y reparacion', 'KernelIA-Maintenance-Operator'),
('sp_security', 'security', 'Security', 'Postura de seguridad local', 'KernelIA-Security-Sentinel'),
('sp_drivers', 'drivers', 'Drivers', 'Dispositivos y controladores', 'KernelIA-Driver-Engineer'),
('sp_filesystem', 'filesystem', 'Filesystem', 'Archivos, carpetas y operaciones de disco', 'KernelIA-Filesystem-Operator'),
('sp_audit', 'audit', 'Audit', 'Logs, auditoria y evidencia', 'KernelIA-Audit-Analyst'),
('sp_performance', 'performance', 'Performance', 'Rendimiento y energia', 'KernelIA-Performance-Tuner'),
('sp_software', 'software', 'Software', 'Ciclo de vida de software instalado', 'KernelIA-Software-Lifecycle'),
('sp_sensitive_ops', 'sensitive_ops', 'Sensitive Ops', 'Acciones administrativas sensibles', 'KernelIA-Sensitive-Executor'),
('sp_megaboss', 'megaboss', 'MegaBoss', 'Operaciones criticas excepcionales', 'KernelIA-MegaBoss-CriticalOps')
ON CONFLICT(id) DO NOTHING;

INSERT INTO confidence_policy (
    id, query_category_id, specialty_id, vector_score_weight, lexical_score_weight,
    exact_match_bonus, specialty_match_bonus, live_state_bonus, tool_verifiability_bonus,
    ambiguity_penalty, short_query_penalty, conflict_penalty, high_threshold, medium_threshold
) VALUES
('cp_short_default', 'qc_short', 'sp_network', 0.0, 1.0, 0.15, 0.10, 0.10, 0.10, 0.20, 0.25, 0.20, 0.85, 0.60),
('cp_specific_default', 'qc_specific', 'sp_network', 0.0, 1.0, 0.20, 0.15, 0.15, 0.15, 0.05, 0.00, 0.10, 0.85, 0.65),
('cp_symptom_default', 'qc_symptom_based', 'sp_performance', 0.0, 1.0, 0.10, 0.15, 0.20, 0.15, 0.10, 0.05, 0.15, 0.82, 0.60)
ON CONFLICT(id) DO NOTHING;

INSERT INTO decision_policy (
    id, query_category_id, specialty_id, confidence_min, risk_max_auto, decision_mode,
    requires_clarification, requires_live_state, requires_snapshot, requires_human, response_style
) VALUES
('dp_short_clarify', 'qc_short', 'sp_network', 0.00, 'r0', 'clarify', 1, 0, 0, 0, 'technical'),
('dp_specific_explain', 'qc_specific', 'sp_network', 0.65, 'r1', 'explain', 0, 1, 0, 0, 'technical'),
('dp_action_simulate', 'qc_action_request', 'sp_sensitive_ops', 0.75, 'r1', 'simulate', 0, 1, 1, 1, 'technical')
ON CONFLICT(id) DO NOTHING;

INSERT INTO clarification_template (id, specialty_id, query_category_id, template_text, target_slot) VALUES
('ct_network_short', 'sp_network', 'qc_short', 'Cuando mencionas red, ¿el problema es DNS, WiFi, gateway, lentitud o falta total de internet?', 'network_problem_type'),
('ct_performance_short', 'sp_performance', 'qc_short', 'Cuando dices que el equipo esta lento, ¿ves CPU alta, disco al 100 o falta de memoria?', 'performance_bottleneck'),
('ct_services_ambiguous', 'sp_services', 'qc_ambiguous', '¿Que servicio de Windows esta fallando exactamente? Si lo sabes, dime el nombre del servicio.', 'service_name')
ON CONFLICT(id) DO NOTHING;
