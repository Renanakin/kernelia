-- Seed de Expresiones Coloquiales de Usuarios No Técnicos (Kernelia RAG v1.8)

INSERT OR IGNORE INTO knowledge_document (id, specialty_id, doc_type, title, slug, summary, body_markdown, source_kind, source_path, version, status, content_hash, created_at, updated_at)
VALUES (
    'doc-non-tech-utterances-master',
    'sp_general',
    'utterance_seed',
    'Banco de Expresiones y Frases de Usuarios No Técnicos',
    'banco-preguntas-no-tecnicas-windows',
    'Catálogo de más de 500 expresiones informales, monosílabos y frases sin contexto para matching agéntico Local-First',
    'Banco de preguntas de usuarios no técnicos para NLU y FTS5',
    'markdown',
    'docs/preguntas_notecnicas.md',
    '1',
    'published',
    'hash-non-tech-500',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- 1. Encendido, Bloqueos y Fallas de Arranque
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-boot-001',
    'doc-non-tech-utterances-master',
    0,
    'Frases de usuarios no técnicos para Encendido y Bloqueos:
mi pc no prende, mi computador no enciende que hago, aprieto el boton y no pasa nada, el pc murio no prende, el notebook no quiere prender, no arranca el windows, se queda pegado al iniciar, se queda en el logo de windows, se queda en la pantalla con el circulito, prende pero se queda en negro, prende y se reinicia solo, el pc se apaga solo de la nada, se congela y no responde, queda congelado todo, no puedo mover nada queda duro, no responde el teclado ni el mouse, dice no boot device, aparece que no hay disco para arrancar, se queda en un menu negro con letras blancas, boton power no hace nada, todo negro al prender.',
    'sp_maintenance',
    'boot_freeze',
    'Frases Coloquiales: Encendido, Bloqueo y Arranque',
    1.5,
    1.5,
    'r0'
);

-- 2. Pantalla, Video, Monitores y Glitches
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-disp-001',
    'doc-non-tech-utterances-master',
    1,
    'Frases de usuarios no técnicos para Pantalla y Video:
no se ve nada en la pantalla, mi pantalla esta negra, monitor negro, dice sin señal en el monitor, la pantalla dice no signal, la pantalla se apago sola, se ve todo raro en la pantalla, pantalla parpadea, todo se ve borroso, se ve gigante todo, todo se ve enorme de la nada, letras enanas no alcanzo a ver, la pantalla se ve muy oscura, no me toma el segundo monitor, no detecta la pantalla externa, el segundo monitor queda negro, no veo la barra de abajo, desaparecio la barra de tareas, pantalla al reves, se dio vuelta la pantalla, salen lineas en la pantalla, solo veo rallas.',
    'sp_display',
    'display_glitch',
    'Frases Coloquiales: Pantalla, Video y Monitores',
    1.5,
    1.5,
    'r0'
);

-- 3. Lentitud, Congelamiento y Rendimiento
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-perf-001',
    'doc-non-tech-utterances-master',
    2,
    'Frases de usuarios no técnicos para Lentitud y Rendimiento:
mi pc esta muy lento, el computador anda muy lento, esta lentisimo, lento lento lento, se demora una eternidad en abrir las cosas, se demora mil años en prender, queda pegado cuando abro un programa, cada vez que abro chrome se pega, cuando abro word se queda colgado, el mouse se mueve a tirones, tarda en reaccionar al hacer clic, disco al 100% todo el rato, el ventilador suena mucho y se traba, va a camara lenta, se cuelga, no responde, no reacciona, pegado, colgado, lag, se frena.',
    'sp_performance',
    'slowness_lag',
    'Frases Coloquiales: Lentitud y Rendimiento',
    1.5,
    1.5,
    'r0'
);

-- 4. Teclado, Mouse y Entrada de Datos
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-input-001',
    'doc-non-tech-utterances-master',
    3,
    'Frases de usuarios no técnicos para Teclado y Mouse:
no funciona el teclado, el teclado no escribe, apreto y no sale nada, el teclado escribe raro, escribo y salen otras letras, no anda el teclado numerico, algunas teclas no funcionan, se cambio el idioma del teclado, la ñ no sale, el mouse no se mueve, el puntero salta solo, el mouse se vuelve loco, mouse wireless no conecta, no puedo hacer clic, touchpad no responde, el pad del notebook no funciona, sin mouse, sin teclado, teclas locas, todo se selecciona solo.',
    'sp_hardware',
    'keyboard_mouse_input',
    'Frases Coloquiales: Teclado, Mouse y Touchpad',
    1.5,
    1.5,
    'r0'
);

-- 5. Audio, Micrófono, Videollamadas y Sonido
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-audio-001',
    'doc-non-tech-utterances-master',
    4,
    'Frases de usuarios no técnicos para Audio y Micrófono:
no se escucha nada, mi pc no tiene sonido, no tengo audio, no suena nada, los audifonos no se escuchan, se escucha muy bajo, se escucha cortado, se escucha como robot, no me escuchan en la reunion, no funciona el microfono, el microfono no toma, dice que no hay dispositivo de audio, audio desaparecio de la nada, silencio, sin sonido, mute, muted, me cambio solo el audio, eco, chirrido, no escucho youtube.',
    'sp_audio',
    'sound_mic',
    'Frases Coloquiales: Audio, Sonido y Micrófono',
    1.5,
    1.5,
    'r0'
);

-- 6. USB, Discos Externos y Gestión de Archivos
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-storage-001',
    'doc-non-tech-utterances-master',
    5,
    'Frases de usuarios no técnicos para USB y Archivos:
conecto el pendrive y no sale, el pendrive no aparece, windows no ve el usb, no me reconoce el pendrive, dice que tengo que formatear el pendrive, no me reconoce el disco externo, me dice acceso denegado, desaparecieron mis archivos, perdi una carpeta, archivo desaparecido, usb muerto, no abre el archivo, dice archivo dañado, dice archivo corrupto, se llena el disco, dice disco lleno, sin espacio en disco.',
    'sp_services',
    'usb_storage_files',
    'Frases Coloquiales: USB, Discos y Archivos',
    1.5,
    1.5,
    'r1'
);

-- 7. Impresoras, Escáner y Cola de Impresión
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-print-001',
    'doc-non-tech-utterances-master',
    6,
    'Frases de usuarios no técnicos para Impresoras y Escáner:
no imprime, mi impresora no imprime, mando a imprimir y no pasa nada, queda en cola de impresion, cola de impresion atascada, impresora sin conexion, muestra un triangulo amarillo, dice error al imprimir, imprime con rayas, sale la hoja en blanco, se traga el papel, papel atascado, manda a otra impresora, escaner no funciona, no escanea, error escaner, escaner no responde.',
    'sp_services',
    'printer_scanner',
    'Frases Coloquiales: Impresoras y Escáneres',
    1.5,
    1.5,
    'r1'
);

-- 8. Inicio de Sesión, Contraseñas y Bloqueos de Usuario
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-auth-001',
    'doc-non-tech-utterances-master',
    7,
    'Frases de usuarios no técnicos para Inicio de Sesión:
no puedo entrar al pc, no puedo iniciar sesion, no me deja poner la clave, dice contraseña incorrecta, dice clave incorrecta, me olvide la contraseña del pc, me olvide la clave del windows, cuenta bloqueada, se queda en bienvenido, me saca de la sesion solo, sale otra cuenta para entrar, pide pin y no lo se, entra pero no se ve mi escritorio normal, sale un perfil temporal.',
    'sp_software',
    'login_account',
    'Frases Coloquiales: Inicio de Sesión y Claves',
    1.5,
    1.5,
    'r0'
);

-- 9. Aplicaciones, Errores y Actualizaciones de Windows
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-app-001',
    'doc-non-tech-utterances-master',
    8,
    'Frases de usuarios no técnicos para Aplicaciones y Windows:
no abre el word, no abre excel, no abre ningun programa, las apps se cierran solas, me sale un mensaje de error al abrir, no me deja instalar nada, no puedo instalar un programa, sale un cuadro con un codigo raro, no abre chrome, explorer se cae, ventanas se quedan blancas, dice no responde, error dll, falta archivo raro, pide permisos de administrador, ventanas emergentes, popups, se queda en actualizando, error de actualizacion.',
    'sp_software',
    'apps_windows_errors',
    'Frases Coloquiales: Aplicaciones y Errores',
    1.5,
    1.5,
    'r0'
);

-- 10. Conectividad a Internet y Modo Avión
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-net-001',
    'doc-non-tech-utterances-master',
    9,
    'Frases de usuarios no técnicos para Internet y Redes:
no tengo internet en el pc, el pc dice sin conexion, sale el icono con una cruz roja, no se conecta al wifi, no me deja poner la clave del wifi, se desconecta el wifi solo, wifi se cae, las paginas no cargan, dice que no hay conexion segura, error de certificado, sin internet, red no identificada, vpn prendida y no entra a nada, modo avion se activa solo, no puedo quitar modo avion.',
    'sp_network',
    'internet_wifi',
    'Frases Coloquiales: Internet y Wi-Fi',
    1.5,
    1.5,
    'r0'
);

-- 11. Pantallazos, Licencias y Errores Raros
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-nontech-bsod-001',
    'doc-non-tech-utterances-master',
    10,
    'Frases de usuarios no técnicos para BSOD y Errores Raros:
me salio una pantalla azul, me salio un pantallazo azul, azul y carita triste, error 0x000 algo, me salen ventanas raras, aparecieron mensajes que no entiendo, desaparecieron iconos, fondo negro, dice que windows no esta activado, sale mensaje windows no original, pantallazo, codigo raro, windows roto, windows dañado, se queda en preparando reparacion automatica, se queda en reparando disco.',
    'sp_drivers',
    'bsod_strange_errors',
    'Frases Coloquiales: Pantallazos Azules y Errores Raros',
    1.5,
    1.5,
    'r1'
);
