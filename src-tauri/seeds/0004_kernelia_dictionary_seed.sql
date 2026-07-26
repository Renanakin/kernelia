-- Seed de Diccionario Maestro de Enrutamiento Agéntico (Kernelia RAG v1.2)

INSERT OR IGNORE INTO knowledge_document (id, title, category, source_type, file_path, metadata_json, created_at, updated_at)
VALUES (
    'doc-dict-master-001',
    'Diccionario Maestro de Enrutamiento Agéntico',
    'System',
    'dict_seed',
    'docs/DICCIONARIO.MD',
    '{"author":"KernelIA Team","type":"routing_dictionary"}',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Chunk Redes y Conectividad
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, title, content, token_count, created_at)
VALUES (
    'chunk-dict-net-001',
    'doc-dict-master-001',
    0,
    'Diccionario: Redes y Conectividad (NetworkAgent)',
    'Palabras clave e intenciones de red: sin internet, red caida, caido, wifi, sin wifi, se fue el wifi, ip, mi ip, gateway, dns, ping, adaptador de red. Especialista: NetworkAgent. Gobernanza: R0 (Diagnóstico no destructivo).',
    42,
    CURRENT_TIMESTAMP
);

-- Chunk Servicios de Windows
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, title, content, token_count, created_at)
VALUES (
    'chunk-dict-svc-001',
    'doc-dict-master-001',
    1,
    'Diccionario: Servicios de Windows (ServicesAgent)',
    'Palabras clave e intenciones de servicios: impresora, no imprime, atascado, spooler, cola de impresion, instalacion de update pegado, iis caido, pagina caida, bits, servicio detenido. Especialista: ServicesAgent. Gobernanza: R2/R3 (Requiere compuerta HITL CHK-XXXX).',
    46,
    CURRENT_TIMESTAMP
);

-- Chunk Procesos y Rendimiento
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, title, content, token_count, created_at)
VALUES (
    'chunk-dict-perf-001',
    'doc-dict-master-001',
    2,
    'Diccionario: Procesos y Rendimiento (PerformanceAgent)',
    'Palabras clave e intenciones de rendimiento: lento, se congela, colgado, cpu 100%, mucho uso, ram alta, pantalla azul, bsod, se apago, se me reinicia, kernel power event 41. Especialista: ProcessAgent y PerformanceAgent. Gobernanza: R1 (Lectura de procesos y eventos).',
    48,
    CURRENT_TIMESTAMP
);

-- Chunk Controladores y Dispositivos
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, title, content, token_count, created_at)
VALUES (
    'chunk-dict-drv-001',
    'doc-dict-master-001',
    3,
    'Diccionario: Controladores y Dispositivos (DriversAgent)',
    'Palabras clave e intenciones de controladores: sin sonido, audio, sin audio, codigo 43, error usb, driver malo, webcam muerta, camara, gpu. Especialista: DriversAgent. Gobernanza: R0/R1 (Diagnóstico de dispositivos).',
    40,
    CURRENT_TIMESTAMP
);

-- Chunk Mantenimiento e Integridad
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, title, content, token_count, created_at)
VALUES (
    'chunk-dict-maint-001',
    'doc-dict-master-001',
    4,
    'Diccionario: Mantenimiento e Integridad (MaintenanceAgent)',
    'Palabras clave e intenciones de mantenimiento: limpieza, basura, optimizar, sfc, archivos danados, reparar, dism, integridad de sistema. Especialista: MaintenanceAgent. Gobernanza: R1/R2 (Integridad de archivos y salud).',
    42,
    CURRENT_TIMESTAMP
);

-- Chunk Almacenamiento y Archivos
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, title, content, token_count, created_at)
VALUES (
    'chunk-dict-fs-001',
    'doc-dict-master-001',
    5,
    'Diccionario: Almacenamiento y Archivos (FilesystemAgent)',
    'Palabras clave e intenciones de archivos: espacio, disco lleno, c lleno, sin permisos, acceso denegado, escritorio, disco c, disco d. Especialista: FilesystemAgent. Gobernanza: R0/R1 (Inventario de particiones y permisos NTFS).',
    42,
    CURRENT_TIMESTAMP
);

-- Chunk Software y Actualizaciones
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, title, content, token_count, created_at)
VALUES (
    'chunk-dict-sw-001',
    'doc-dict-master-001',
    6,
    'Diccionario: Software y Actualizaciones (SoftwareAgent)',
    'Palabras clave e intenciones de software: instalar, winget, programa nuevo, que hay instalado, inventario software, windows update, parches. Especialista: SoftwareAgent. Gobernanza: R1/R2 (Inspección y gestión de software).',
    42,
    CURRENT_TIMESTAMP
);
