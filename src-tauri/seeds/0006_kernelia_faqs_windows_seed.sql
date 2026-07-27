-- Seed de 100 FAQs de Windows Cliente Nivel Usuario (Kernelia RAG Local-First v1.6)

INSERT OR IGNORE INTO knowledge_document (id, specialty_id, doc_type, title, slug, summary, body_markdown, source_kind, source_path, version, status, content_hash, created_at, updated_at)
VALUES (
    'doc-faqs-win-master',
    'sp_general',
    'faq_seed',
    'Base de Conocimiento 100 FAQs Windows Nivel Usuario',
    'faqs-windows-nivel-usuario-100',
    '100 Preguntas y Respuestas frecuentes orientadas a uso de Windows 10/11, inicio de sesión, escritorio y resolución de fallas comunes',
    'Base de Conocimiento 100 FAQs Windows Nivel Usuario para respuestas Local-First instantáneas',
    'markdown',
    'docs/faqs_windows.md',
    '1',
    'published',
    'hash-faqs-win-100',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Chunk 1: Inicio de Sesión, Cuentas y Perfil de Usuario
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-win-auth-001',
    'doc-faqs-win-master',
    0,
    'FAQs Windows - Cuentas e Inicio de Sesión:
- ¿No puedo iniciar sesión en Windows?: Revisa usuario, contraseña y que el teclado esté en el idioma adecuado.
- ¿Cuenta bloqueada?: Comprueba si se debe a demasiados intentos y espera unos minutos.
- ¿Olvidé contraseña?: Si es cuenta local usa recuperación; si es corporativa sigue el flujo de TI o cuenta Microsoft en línea.
- ¿Problemas con el perfil de usuario?: Ve a Configuración > Cuentas. Si falla solo un perfil, crea un usuario de prueba para aislar la falla.',
    'sp_software',
    'user_profile',
    'FAQ Windows: Cuentas y Perfiles de Usuario',
    1.2,
    1.2,
    'r0'
);

-- Chunk 2: Escritorio, Barra de Tareas y Programas de Inicio
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-win-desktop-001',
    'doc-faqs-win-master',
    1,
    'FAQs Windows - Escritorio y Administración de Inicio:
- ¿Desactivar programas que abren al iniciar Windows?: Revisa pestaña Inicio en Administrador de Tareas o Configuración > Aplicaciones > Inicio.
- ¿Escritorio o barra de tareas no responde?: Reinicia el Explorador de Windows (explorer.exe) desde el Administrador de Tareas.
- ¿Windows tarda mucho en arrancar?: Deshabilita programas de inicio innecesarios para reducir tiempo de inicio.',
    'sp_performance',
    'desktop_explorer',
    'FAQ Windows: Escritorio y Programas de Inicio',
    1.2,
    1.2,
    'r0'
);

-- Chunk 3: Almacenamiento, Liberación de Espacio y Archivos Temporales
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-win-storage-001',
    'doc-faqs-win-master',
    2,
    'FAQs Windows - Disco y Almacenamiento:
- ¿Poco espacio en disco?: Ejecuta Liberador de espacio en disco (cleanmgr) y revisa Configuración > Sistema > Almacenamiento.
- ¿Qué carpetas revisar primero?: Revisa Descargas, Papelera de Reciclaje y carpetas de datos temporales.',
    'sp_filesystem',
    'cleanmgr',
    'FAQ Windows: Liberación de Espacio en Disco',
    1.2,
    1.2,
    'r0'
);

-- Chunk 4: Teclado, Idioma, Región y Accesibilidad
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-win-kbd-001',
    'doc-faqs-win-master',
    3,
    'FAQs Windows - Teclado e Idioma:
- ¿Teclado escribe caracteres raros o tildes incorrectos?: Cambia distribución en Configuración > Hora e idioma > Idioma y región (Español Latinoamérica).
- ¿Teclado reacciona raro?: Desactiva "Teclas filtro" u opciones de accesibilidad en Configuración.',
    'sp_hardware',
    'keyboard_layout',
    'FAQ Windows: Idioma y Configuración de Teclado',
    1.2,
    1.2,
    'r0'
);

-- Chunk 5: Audio, Audífonos y Permisos de Micrófono
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-win-audio-001',
    'doc-faqs-win-master',
    4,
    'FAQs Windows - Audio y Dispositivos de Sonido:
- ¿Sin sonido en Windows?: Revisa el volumen general, silenciador y elige el dispositivo de salida correcto en la barra de tareas.
- ¿App no puede usar micrófono?: Concede permisos en Configuración > Privacidad > Micrófono.
- ¿Audífonos Bluetooth no se conectan?: Reconecta el dispositivo en Configuración > Dispositivos Bluetooth.',
    'sp_audio',
    'sound_permissions',
    'FAQ Windows: Audio, Micrófono y Permisos',
    1.2,
    1.2,
    'r0'
);

-- Chunk 6: Glitches Gráficos, Monitores y Modos de Pantalla
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-win-display-001',
    'doc-faqs-win-master',
    5,
    'FAQs Windows - Pantalla y Monitores:
- ¿Pantalla borrosa o letras gigantes?: Ajusta la resolución nativa y el escalado de texto en Configuración > Sistema > Pantalla.
- ¿Glitches gráficos o parpadeo?: Actualiza el controlador de video desde Windows Update o la página del fabricante.
- ¿Configurar dos monitores?: Usa Win + P para seleccionar Extender o Duplicar.',
    'sp_display',
    'screen_resolution',
    'FAQ Windows: Resolución y Glitches Gráficos',
    1.2,
    1.2,
    'r0'
);

-- Chunk 7: Herramientas de Integridad (SFC, DISM), Modo Seguro y Activación
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-faqs-win-diag-001',
    'doc-faqs-win-master',
    6,
    'FAQs Windows - Diagnóstico, Integridad y Seguridad:
- ¿Herramienta básica para reparar errores aleatorios?: Ejecuta `sfc /scannow` en Símbolo del Sistema como Administrador.
- ¿Reparar componentes dañados de Windows?: Ejecuta `DISM /Online /Cleanup-Image /RestoreHealth`.
- ¿Modo Seguro (Safe Mode)?: Úsalo para arrancar sin programas de terceros y desinstalar drivers con fallas.
- ¿Seguridad de Windows (Windows Defender)?: Mantiene protegido el equipo con análisis rápido y completo integrado.
- ¿Windows dice que no está activado?: Revisa la sección Configuración > Actualización y seguridad > Activación.',
    'sp_maintenance',
    'sfc_dism',
    'FAQ Windows: Diagnóstico SFC, DISM y Modo Seguro',
    1.2,
    1.2,
    'r1'
);
