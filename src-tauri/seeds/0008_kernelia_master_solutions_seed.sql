-- Seed de Soluciones Maestras y Contexto Completo para Clasificación Perfecta (Kernelia RAG v1.9)

INSERT OR IGNORE INTO knowledge_document (id, specialty_id, doc_type, title, slug, summary, body_markdown, source_kind, source_path, version, status, content_hash, created_at, updated_at)
VALUES (
    'doc-master-solutions-perfect-score',
    'sp_general',
    'solution_catalog_seed',
    'Catálogo Maestro de Soluciones Agénticas de Puesto de Trabajo y Hardware',
    'catalogo-maestro-soluciones-soporte-perfect-score',
    'Soluciones estructuradas paso a paso con diagnostico hardware, procedimientos no destructivos y recomendaciones para el 100% de consultas',
    'Catálogo Maestro de Soluciones Agénticas Kernelia RAG v1.9',
    'markdown',
    'docs/preguntas_notecnicas.md',
    '1',
    'published',
    'hash-master-solutions-008',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Solución 1: PC no enciende / Botón Power / Falla Eléctrica Hardware
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-power-001',
    'doc-master-solutions-perfect-score',
    0,
    'Problema: El computador o notebook no enciende, boton power no hace nada, PC muerto.
### Solución
1. Verifique que el cable de alimentación o cargador esté firmemente conectado a la toma eléctrica y al equipo.
2. Si es notebook, desconecte la batería y el cargador, mantenga presionado el botón de encendido por 30 segundos (drenaje de energía estática) y vuelva a conectar.
3. Pruebe otro enchufe de pared y verifique si el LED indicador de carga o la fuente de poder emiten luz.
4. Si es PC de escritorio, confirme que el interruptor I/O de la fuente de poder trasera esté en posición I (Encendido).

### Consejos y Recomendaciones
- Evite usar extensiones eléctricas o zapatillas sobrecargadas sin supresor de picos.
- Si el equipo huele a quemado o no emite sonido de ventiladores, escale a soporte físico para revisión de la fuente ATX o placa madre.',
    'sp_maintenance',
    'hardware_power',
    'Solución Maestra: Encendido y Alimentación Eléctrica',
    2.0,
    2.0,
    'r0'
);

-- Solución 2: Pantalla Negra / Sin Señal / Glitches de Video / Pantalla al Revés
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-disp-001',
    'doc-master-solutions-perfect-score',
    1,
    'Problema: Pantalla negra, sin señal, se dio vuelta la pantalla, parpadeo o letras borrosas.
### Solución
1. Si la pantalla se dio vuelta, presione la combinación de teclas `Ctrl + Alt + Flecha Arriba` para restaurar la orientación vertical.
2. Si el monitor dice "Sin Señal", verifique la firmeza del cable (HDMI, DisplayPort o VGA) y la fuente de entrada seleccionada (Input/Source en el menú del monitor).
3. Presione `Win + Ctrl + Shift + B` para reiniciar el controlador gráfico de Windows en caliente.
4. Presione `Win + P` para seleccionar la opción "Duplicar" o "Extender" si utiliza un segundo monitor.

### Consejos y Recomendaciones
- Revisa que la resolución configurada en Configuración > Pantalla coincida con la resolución recomendada nativa del monitor.
- Si la imagen parpadea o muestra líneas de colores, pruebe con otro cable de video para descartar falla física del cable.',
    'sp_display',
    'hardware_display',
    'Solución Maestra: Pantalla, Video y Orientación',
    2.0,
    2.0,
    'r0'
);

-- Solución 3: Equipo Lentísimo / Congelado / Ventilador a Full / Disco al 100%
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-perf-001',
    'doc-master-solutions-perfect-score',
    2,
    'Problema: Equipo lentísimo, congelado, va a cámara lenta, disco al 100%, ventilador ruidoso.
### Solución
1. Presione `Ctrl + Shift + Esc` para abrir el Administrador de tareas, ordene los procesos por uso de CPU/RAM/Disco e identifique aplicaciones de alto consumo.
2. Ejecute el Liberador de espacio en disco de Windows (`cleanmgr`) para eliminar archivos temporales y caché del sistema.
3. Verifique en Administrador de tareas > pestaña "Inicio" y deshabilite programas innecesarios que se ejecuten automáticamente al encender.
4. Si el disco mecánico o SSD muestra consumo del 100% continuo, confirme que Windows Update no esté descargando parches en segundo plano.

### Consejos y Recomendaciones
- Mantenga las salidas de aire del equipo libres de polvo para prevenir que la elevación de temperatura fuerce al procesador a bajar su velocidad (Thermal Throttling).
- Reinicie el equipo al menos una vez por semana para liberar memoria RAM acumulada por procesos residuales.',
    'sp_performance',
    'hardware_performance',
    'Solución Maestra: Rendimiento, Temperatura y Limpieza',
    2.0,
    2.0,
    'r0'
);

-- Solución 4: Teclado escribe raro / Mouse loco / Touchpad no responde
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-input-001',
    'doc-master-solutions-perfect-score',
    3,
    'Problema: Teclado escribe caracteres raros, la ñ no sale, el mouse no se mueve, touchpad bloqueado.
### Solución
1. Para corregir la disposición del teclado, presione `Alt + Shift` o `Win + Espacio` para cambiar al idioma "Español (Latinoamérica)".
2. Si el touchpad del notebook no responde, presione la tecla de función dedicada de bloqueo (ej. `Fn + F6`, `Fn + F7` o `Fn + F9` según la marca).
3. Si el mouse inalámbrico o teclado Bluetooth falla, desconecte el receptor USB, reemplace la batería y conéctelo en otro puerto USB directo.
4. Deshabilite "Teclas filtro" en Configuración > Accesibilidad > Teclado si las teclas tardan en reaccionar al escribir.

### Consejos y Recomendaciones
- Pruebe el dispositivo en otro PC para determinar si la falla es de hardware físico o de configuración de Windows.
- En notebooks compactos, asegúrese de que la tecla `Bloq Num` (Num Lock) no esté activada por error.',
    'sp_hardware',
    'hardware_peripherals',
    'Solución Maestra: Periféricos de Entrada y Teclado',
    2.0,
    2.0,
    'r0'
);

-- Solución 5: Sin Audio / Se escucha como robot / Micrófono silenciado
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-audio-001',
    'doc-master-solutions-perfect-score',
    4,
    'Problema: Sin sonido, audífonos no suenan, micrófono no toma en videollamadas, audio distorsionado.
### Solución
1. Haga clic en el ícono del altavoz en la barra de tareas y confirme que el dispositivo de salida correcto (Altavoces, Audífonos o HDMI) esté seleccionado.
2. Vaya a Configuración > Privacidad y seguridad > Micrófono y asegúrese de que la opción "Permitir que las aplicaciones accedan al micrófono" esté activada.
3. Si el audio suena distorsionado o como robot en videollamadas, cierre otras aplicaciones que consuman ancho de banda o reinicie el servicio `Audiosrv` en `services.msc`.
4. Verifique el botón físico de silenciador (Mute) en el cable de los audífonos o en el teclado.

### Consejos y Recomendaciones
- Si utiliza audífonos Bluetooth, desvincúlelos y vuelva a emparejarlos en Configuración > Dispositivos.
- Ejecute el Solucionador de problemas de audio integrado en Configuración > Sistema > Solucionar problemas.',
    'sp_audio',
    'hardware_audio',
    'Solución Maestra: Dispositivos de Sonido y Micrófono',
    2.0,
    2.0,
    'r0'
);

-- Solución 6: Pendrive o Disco Externo no aparece / Pide formatear / Acceso Denegado
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-storage-001',
    'doc-master-solutions-perfect-score',
    5,
    'Problema: Pendrive o disco duro externo no aparece en el explorador, pide formatear, acceso denegado.
### Solución
1. Presione `Win + X` y seleccione "Administración de discos" (`diskmgmt.msc`). Si la unidad aparece sin letra asignada, haga clic derecho y seleccione "Cambiar la letra y rutas de acceso a la unidad".
2. Si Windows pide formatear la unidad, **no la formatee** para evitar pérdida de datos. Ejecute `chkdsk X: /f` en la consola de comandos (reemplazando X por la letra de la unidad).
3. Conecte el pendrive en un puerto USB posterior del PC (directo a la placa madre) para asegurar suficiente alimentación eléctrica.
4. Si indica "Acceso denegado", revise los permisos de seguridad de la carpeta en Propiedades > Seguridad.

### Consejos y Recomendaciones
- Use la opción "Quitar hardware de forma segura" antes de desconectar memorias USB para evitar corrupción de archivos.
- Evite usar concentradores (hubs) USB sin alimentación propia para discos externos mecánicos de 2.5 pulgadas.',
    'sp_services',
    'hardware_storage',
    'Solución Maestra: Almacenamiento USB y Permisos de Disco',
    2.0,
    2.0,
    'r1'
);

-- Solución 7: Impresora no imprime / Cola atascada / Papel trabado
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-print-001',
    'doc-master-solutions-perfect-score',
    6,
    'Problema: Impresora no imprime, documentos atascados en cola, triangulo amarillo, papel trabado.
### Solución
1. Para limpiar trabajos atascados en la cola de impresión, abra PowerShell como Administrador y ejecute:
   `Stop-Service Spooler; Remove-Item $env:SystemRoot\System32\Spool\PRINTERS\* -Force; Start-Service Spooler`
2. Verifique en Panel de control > Dispositivos e Impresoras que la impresora esté marcada como "Predeterminada" y su estado sea "En línea".
3. En caso de atasco físico de papel, apague la impresora y retire el papel suavemente en la dirección del flujo de impresión sin forzar los rodillos.
4. Si la impresora se conecta por USB, desconecte y vuelva a conectar el cable USB a un puerto activo del PC.

### Consejos y Recomendaciones
- Asegúrese de utilizar controladores oficiales del fabricante y no drivers genéricos de prueba si requiere funciones avanzadas de impresión.
- Valide que el papel utilizado esté completamente seco y no arrugado para evitar nuevos atascos.',
    'sp_services',
    'hardware_printer',
    'Solución Maestra: Impresoras, Cola Spooler y Atascos',
    2.0,
    2.0,
    'r1'
);

-- Solución 8: Olvidé contraseña / Cuenta bloqueada / Perfil temporal
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-auth-001',
    'doc-master-solutions-perfect-score',
    7,
    'Problema: Olvidé la contraseña de Windows, cuenta bloqueada, pantalla de inicio en perfil temporal.
### Solución
1. Si utiliza una cuenta Microsoft (@outlook.com/@hotmail.com/@live.com), restablezca la contraseña desde otro dispositivo ingresando a `account.live.com/password/reset`.
2. Si la cuenta está bloqueada por demasiados intentos fallidos, espere 15 a 30 minutos sin intentar iniciar sesión para que el bloqueo de seguridad se libere automáticamente.
3. Si Windows inicia en un "Perfil temporal" (archivos y fondo desaparecidos), reinicie el equipo 3 veces consecutivas para permitir que Windows recupere el registro de perfil original.
4. Verifique que la tecla `Mayús` (Shift) o `Bloq Mayús` no esté activada al escribir la clave.

### Consejos y Recomendaciones
- En entornos corporativos o de dominio, contacte al administrador de TI para el desbloqueo de cuenta Active Directory.
- Configure opciones de recuperación de PIN o preguntas de seguridad en Configuración > Cuentas > Opciones de inicio de sesión.',
    'sp_software',
    'hardware_auth',
    'Solución Maestra: Cuentas, Claves y Recuperación de Perfil',
    2.0,
    2.0,
    'r0'
);

-- Solución 9: Sin Internet / Wi-Fi se desconecta / Modo Avión atascado
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-net-001',
    'doc-master-solutions-perfect-score',
    8,
    'Problema: Sin conexión a internet, icono con cruz roja, Wi-Fi se desconecta solo, Modo Avión atascado.
### Solución
1. Si el Modo Avión no se desactiva, presione la combinación de teclas de hardware `Fn + F2` o `Fn + F12` (según el fabricante del notebook).
2. Abra la consola de comandos CMD como Administrador y ejecute la secuencia de limpieza de red:
   `ipconfig /flushdns`
   `netsh winsock reset`
3. Reinicie el adaptador de red desde Configuración > Red e Internet > Configuración de red avanzada > Restablecimiento de red.
4. Reinicie el módem o router principal desconectando el cable de corriente durante 10 segundos.

### Consejos y Recomendaciones
- Si el ícono muestra "Red no identificada", deshabilite conexiones VPN activas temporalmente para descartar bloqueos de túnel.
- Compruebe que la fecha y hora de Windows estén sincronizadas correctamente, ya que una hora incorrecta bloquea la navegación HTTPS.',
    'sp_network',
    'hardware_network',
    'Solución Maestra: Redes, Wi-Fi y Conectividad',
    2.0,
    2.0,
    'r0'
);

-- Solución 10: Pantallazo Azul (BSOD) / Carita Triste / Error 0x000
INSERT OR IGNORE INTO knowledge_chunk (id, document_id, chunk_index, chunk_text, specialty_id, entity_key, title_anchor, lexical_weight, semantic_weight, risk_level_hint)
VALUES (
    'chunk-sol-bsod-001',
    'doc-master-solutions-perfect-score',
    9,
    'Problema: Pantallazo azul (BSOD), carita triste, error 0x000, Windows no arranca o se reinicia en bucle.
### Solución
1. Anote el código de error en pantalla (ej. `SYSTEM_SERVICE_EXCEPTION`, `CRITICAL_PROCESS_DIED`, `0x80070002`).
2. Si el sistema logra iniciar, ejecute la verificación de archivos corruptos del sistema en CMD como Administrador: `sfc /scannow`.
3. Si el comando SFC encuentra errores, repare la imagen de Windows ejecutando: `DISM /Online /Cleanup-Image /RestoreHealth`.
4. Si el equipo entra en bucle de reinicios, inicie en Modo seguro desde las opciones avanzadas de recuperación y desinstale el último controlador o actualización instalada.

### Consejos y Recomendaciones
- Desconecta periféricos externos no esenciales (camaras web, discos externos, adaptadores USB) para aislar incompatibilidades de hardware.
- Realice un análisis completo de memoria RAM con la herramienta `mdsched.exe` (Diagnóstico de memoria de Windows).',
    'sp_drivers',
    'hardware_bsod',
    'Solución Maestra: Pantallazos Azules y Diagnóstico SFC/DISM',
    2.0,
    2.0,
    'r1'
);
