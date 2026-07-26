-- Seed de Diccionario Maestro de Enrutamiento Agéntico (Kernelia RAG v1.2)

INSERT OR IGNORE INTO knowledge_document (id, specialty_id, doc_type, title, slug, summary, body_markdown, source_kind, source_path, version, status, content_hash, created_at, updated_at)
VALUES (
    'doc-dict-master-001',
    'sp_general',
    'dict_seed',
    'Diccionario Maestro de Enrutamiento Agéntico',
    'diccionario-maestro-enrutamiento-agentico',
    'Diccionario de términos coloquiales, técnicos y monosílabos para Kernelia RAG',
    'Diccionario Maestro de Enrutamiento Agéntico Kernelia RAG v1.2',
    'markdown',
    'docs/DICCIONARIO.MD',
    '1',
    'published',
    'hash-dict-001',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Chunk Redes y Conectividad
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-dict-net-001',
    'doc-dict-master-001',
    0,
    'Palabras clave e intenciones de red: sin internet, red caida, caido, wifi, sin wifi, se fue el wifi, ip, mi ip, gateway, dns, ping, adaptador de red. Especialista: NetworkAgent. Gobernanza: R0 (Diagnóstico no destructivo).',
    'sp_network',
    'net',
    'Diccionario: Redes y Conectividad (NetworkAgent)',
    1.0,
    1.0,
    'r0'
);

-- Chunk Servicios de Windows
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-dict-svc-001',
    'doc-dict-master-001',
    1,
    'Palabras clave e intenciones de servicios: impresora, no imprime, atascado, spooler, cola de impresion, instalacion de update pegado, iis caido, pagina caida, bits, servicio detenido. Especialista: ServicesAgent. Gobernanza: R2/R3 (Requiere compuerta HITL CHK-XXXX).',
    'sp_services',
    'spooler',
    'Diccionario: Servicios de Windows (ServicesAgent)',
    1.0,
    1.0,
    'r2'
);

-- Chunk Procesos y Rendimiento
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-dict-perf-001',
    'doc-dict-master-001',
    2,
    'Palabras clave e intenciones de rendimiento: lento, se congela, colgado, cpu 100%, mucho uso, ram alta, pantalla azul, bsod, se apago, se me reinicia, kernel power event 41. Especialista: ProcessAgent y PerformanceAgent. Gobernanza: R1 (Lectura de procesos y eventos).',
    'sp_performance',
    'performance',
    'Diccionario: Procesos y Rendimiento (PerformanceAgent)',
    1.0,
    1.0,
    'r1'
);

-- Chunk Controladores y Dispositivos
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-dict-drv-001',
    'doc-dict-master-001',
    3,
    'Palabras clave e intenciones de controladores: sin sonido, audio, sin audio, codigo 43, error usb, driver malo, webcam muerta, camara, gpu. Especialista: DriversAgent. Gobernanza: R0/R1 (Diagnóstico de dispositivos).',
    'sp_drivers',
    'gpu',
    'Diccionario: Controladores y Dispositivos (DriversAgent)',
    1.0,
    1.0,
    'r1'
);

-- Chunk Mantenimiento e Integridad
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-dict-maint-001',
    'doc-dict-master-001',
    4,
    'Palabras clave e intenciones de mantenimiento: limpieza, basura, optimizar, sfc, archivos danados, reparar, dism, integridad de sistema. Especialista: MaintenanceAgent. Gobernanza: R1/R2 (Integridad de archivos y salud).',
    'sp_maintenance',
    'sfc',
    'Diccionario: Mantenimiento e Integridad (MaintenanceAgent)',
    1.0,
    1.0,
    'r1'
);

-- Chunk Almacenamiento y Archivos
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-dict-fs-001',
    'doc-dict-master-001',
    5,
    'Palabras clave e intenciones de archivos: espacio, disco lleno, c lleno, sin permisos, acceso denegado, escritorio, disco c, disco d. Especialista: FilesystemAgent. Gobernanza: R0/R1 (Inventario de particiones y permisos NTFS).',
    'sp_filesystem',
    'disk',
    'Diccionario: Almacenamiento y Archivos (FilesystemAgent)',
    1.0,
    1.0,
    'r0'
);

-- Chunk Software y Actualizaciones
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-dict-sw-001',
    'doc-dict-master-001',
    6,
    'Palabras clave e intenciones de software: instalar, winget, programa nuevo, que hay instalado, inventario software, windows update, parches. Especialista: SoftwareAgent. Gobernanza: R1/R2 (Inspección y gestión de software).',
    'sp_software',
    'windows_update',
    'Diccionario: Software y Actualizaciones (SoftwareAgent)',
    1.0,
    1.0,
    'r1'
);
