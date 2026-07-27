-- Seed de 100 FAQs de Mesa de Ayuda TI (Kernelia RAG Local-First v1.5)

INSERT OR IGNORE INTO knowledge_document (id, specialty_id, doc_type, title, slug, summary, body_markdown, source_kind, source_path, version, status, content_hash, created_at, updated_at)
VALUES (
    'doc-faqs-100-master',
    'sp_general',
    'faq_seed',
    'Base de Conocimiento 100 FAQs Mesa de Ayuda TI',
    'faqs-mesa-ayuda-ti-100',
    '100 Preguntas y Respuestas frecuentes sobre soporte técnico de escritorio, periféricos y sistema operativo',
    'Base de Conocimiento 100 FAQs Mesa de Ayuda TI para respuestas Local-First instantáneas',
    'markdown',
    'docs/faqs_mesa_ayuda_ti_100.md',
    '1',
    'published',
    'hash-faqs-100',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Chunk 1: Mantenimiento, Arranque y Encendido del Sistema
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-maint-001',
    'doc-faqs-100-master',
    0,
    'Preguntas frecuentes de Arranque y Encendido:
- ¿El computador no enciende?: Verifica cable de poder, fuente, batería y prueba otro enchufe antes de escalar.
- ¿La pantalla está negra al encender?: Revisa energía del monitor, firmeza del cable de video y entrada seleccionada.
- ¿El equipo se queda pegado o congelado?: Fuerza reinicio y anota si ocurre al abrir una app o al iniciar Windows.
- ¿El notebook no responde al botón de encendido?: Confirma que no esté en suspensión y mantén presionado el botón para reinicio forzado.
- ¿El notebook no carga batería?: Revisa el cargador, puerto, LED de carga y adaptador compatible.
- ¿El equipo se apaga solo o se calienta?: Comprueba sobrecalentamiento, ventilación bloqueada o acumulación de polvo.
- ¿Windows tarda demasiado en arrancar?: Revisa espacio libre, aplicaciones de inicio y actualizaciones del sistema.',
    'sp_maintenance',
    'boot',
    'FAQ 100: Arranque, Encendido y Salud de Hardware',
    1.2,
    1.2,
    'r0'
);

-- Chunk 2: Rendimiento, Consumo de Recursos y Procesos
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-perf-001',
    'doc-faqs-100-master',
    1,
    'Preguntas frecuentes de Rendimiento y Recursos:
- ¿El computador está muy lento?: Cierra procesos pesados, revisa uso de disco, memoria RAM y CPU en Administrador de Tareas.
- ¿Disco al 100%?: Confirma si hay actualizaciones de Windows en curso o análisis de antivirus completo.
- ¿Lento incluso sin abrir programas grandes?: Libera almacenamiento disponible y elimina archivos temporales.
- ¿El ventilador suena mucho?: Revisa procesos atascados o servicios consumiendo CPU en segundo plano.
- ¿Lentitud tras varios días encendido?: Reinicia el equipo para liberar memoria RAM y recursos de Kernel.
- ¿Cómo mejorar rendimiento sin formatear?: Desinstala programas innecesarios y deshabilita apps de inicio automático.',
    'sp_performance',
    'performance',
    'FAQ 100: Rendimiento y Optimización de Recursos',
    1.2,
    1.2,
    'r0'
);

-- Chunk 3: Controladores, Dispositivos y Pantallazos Azules (BSOD)
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-drv-001',
    'doc-faqs-100-master',
    2,
    'Preguntas frecuentes de Drivers y BSOD:
- ¿Aparece una pantalla azul (BSOD)?: Reinicia y verifica si ocurrió tras instalar un driver o programa nuevo. Guarda el código de error.
- ¿Un controlador falla o da advertencia amarilla?: Revisa el Administrador de dispositivos (devmgmt.msc), actualiza o revierte el driver.
- ¿Dónde ver el código de error de un dispositivo?: En las propiedades del dispositivo en Administrador de dispositivos.
- ¿Driver genérico o del fabricante?: Utiliza la versión oficial del fabricante si el genérico presenta inestabilidad.',
    'sp_drivers',
    'bsod',
    'FAQ 100: Controladores y Pantallazos Azules',
    1.2,
    1.2,
    'r1'
);

-- Chunk 4: Periféricos Teclado y Mouse
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-periph-001',
    'doc-faqs-100-master',
    3,
    'Preguntas frecuentes de Teclado y Mouse:
- ¿Teclado o mouse no funciona?: Verifica cable, batería, puerto USB o reempareja Bluetooth. Prueba en otro equipo.
- ¿Teclado escribe en otro idioma o caracteres incorrectos?: Revisa la distribución e idioma del teclado en Configuración regional de Windows.
- ¿Teclado numérico no funciona?: Revisa la tecla Bloq Num (Num Lock) activa.
- ¿Se desconecta tras suspensión?: Reinstala el dispositivo o deshabilita la suspensión de energía en los puertos USB.',
    'sp_hardware',
    'peripherals',
    'FAQ 100: Periféricos Teclado y Mouse',
    1.2,
    1.2,
    'r0'
);

-- Chunk 5: Monitores, Pantallas y Estaciones de Acoplamiento (Docking)
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-disp-001',
    'doc-faqs-100-master',
    4,
    'Preguntas frecuentes de Monitores y Pantalla:
- ¿Segundo monitor no aparece o no da imagen?: Habilítalo en Configuración de pantalla (Win + P) y fuerza detección.
- ¿Imagen desenfocada o cortada?: Cambia la resolución a la nativa recomendada y ajusta la frecuencia de refresco.
- ¿Docking station no detecta pantallas?: Confirma que la estación soporte la cantidad de monitores y valida el cable USB-C / DisplayPort.
- ¿HDMI quitó el audio?: Revisa el dispositivo de reproducción predeterminado en la barra de tareas.',
    'sp_display',
    'display',
    'FAQ 100: Monitores y Pantallas Secundarias',
    1.2,
    1.2,
    'r0'
);

-- Chunk 6: Audio, Micrófono y Videollamadas
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-audio-001',
    'doc-faqs-100-master',
    5,
    'Preguntas frecuentes de Audio y Micrófono:
- ¿No hay sonido?: Revisa el volumen del sistema, salida seleccionada y mute. Confirma conexión de audífonos.
- ¿Micrófono no funciona en videollamadas?: Valida permisos de privacidad del micrófono en Configuración de Windows y silenciador físico.
- ¿El audio cambió de salida solo?: Selecciona manualmente el dispositivo de reproducción correcto en el icono del altavoz.',
    'sp_audio',
    'audio',
    'FAQ 100: Audio, Altavoces y Micrófono',
    1.2,
    1.2,
    'r0'
);

-- Chunk 7: Dispositivos USB e Impresoras de Escritorio
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-print-001',
    'doc-faqs-100-master',
    6,
    'Preguntas frecuentes de USB e Impresión:
- ¿USB o pendrive no aparece?: Prueba otro puerto USB, asigna letra de unidad en Administración de discos (diskmgmt.msc).
- ¿La impresora no imprime o documentos pegados en cola?: Cancela trabajos atascados y reinicia el servicio Cola de impresión (Spooler).
- ¿Imprime en otra impresora?: Selecciona la impresora correcta como predeterminada en Panel de Control.
- ¿Papel atascado?: Sigue la ruta de extracción del papel indicada por el fabricante sin forzar rodillos.',
    'sp_services',
    'printer',
    'FAQ 100: Impresoras y Almacenamiento USB',
    1.2,
    1.2,
    'r1'
);
