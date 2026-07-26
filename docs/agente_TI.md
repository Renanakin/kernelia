Estándar de Ingeniería: Maestro Agéntico de Soporte Windows (ITSM/ITOM)
1. Principio Operativo y Ciclo de Vida del Agente
El sistema abandona el paradigma de los chatbots lineales de pregunta-respuesta y se rige bajo un bucle agéntico autónomo de 5 fases ejecutadas de forma cíclica hasta la resolución o el bloqueo por gobernanza:

Percepción: Captura de la alerta ITOM o el ticket de Service Desk (evento, error o solicitud del usuario).

Razonamiento y Triage: Análisis semántico del problema para clasificar el incidente dentro del ecosistema Windows (Directorio Activo, Subsistema de Red/WinRM, Almacenamiento NTFS o Servicios Win32).

Planificación y Recuperación RAG: Consulta estricta de Runbooks técnicos normalizados (bajo marcos ITIL 4) para evitar alucinaciones operativas.

Ejecución Controlada: Invocación de herramientas o wrappers de automatización sobre el entorno Windows.

Observabilidad y Gobernanza: Evaluación continua del impacto de la acción y validación de seguridad.

2. Taxonomía de Gobernanza y Modelo HITL (Human-in-the-Loop)
Toda acción o herramienta disponible para el agente debe clasificarse obligatoriamente bajo una taxonomía de 5 niveles de riesgo, implementando compuertas de aprobación humanas previas a la ejecución cuando el impacto afecte la continuidad del negocio:

Nivel 0 (Autonomía Total): Diagnósticos de lectura, recolección de logs del visor de eventos de Windows (Get-EventLog), pruebas de conectividad de red básicas.

Nivel 1 (Autonomía Condicionada con Notificación): Limpieza de cachés locales, reinicio de servicios de bajo impacto que no dependen de clústeres core.

Nivel 2 (Aprobación Requerida - HITL Estándar): Reinicio de servicios críticos (como spooler, IIS), reseteo de tokens de sesión o políticas de usuario menores en Directorio Activo.

Nivel 3 (Aprobación Estricta de Supervisor): Modificaciones de permisos NTFS globales, cambios en reglas de Firewall corporativo o alteraciones de GPO (Group Policy Objects).

Nivel 4 (Bloqueo Absoluto / Prohibido): Acciones destructivas no reversibles sobre controladores de dominio primarios o particiones de sistema raíz.

3. Especificaciones Técnicas para el Equipo de Desarrollo
A. Orquestación y Persistencia de Estado
Requisito de Framework: Utilizar motores de grafos de estado cíclicos que admitan persistencia nativa y checkpoints (permitiendo congelar el estado del agente y mantenerlo en pausa mientras un administrador humano aprueba o rechaza una operación en la compuerta HITL).

Gestión de Memoria: El almacenamiento de estado debe aislar el contexto por identificador de ticket (incident_id) para evitar la contaminación cruzada de datos entre incidentes concurrentes.

B. Motor RAG Confiable (Grounding Técnico)
Fuentes Restringidas: El recuperador de conocimiento debe indexar exclusivamente la documentación oficial de sysadmin de Windows, runbooks internos aprobados y bitácoras de incidentes históricos reales resueltos.

Política de Cero Alucinación: Si el motor RAG no encuentra un procedimiento validado para el error específico reportado en Windows, el agente tiene la instrucción estricta de detener la automatización y escalar directamente al operador humano de Nivel 2, prohibiendo la generación sintética de comandos de consola.

C. Capa de Ejecución y Sandboxing (PowerShell Remoto)
Principio de Mínimo Privilegio: Ninguna herramienta de ejecución automatizada sobre Windows puede correr bajo credenciales de Administrador Local permanente. Debe utilizar cuentas de servicio acotadas y específicas para la tarea asignada.

Auditoría Estricta: Cada comando ejecutado (ya sea mediante PowerShell Core o WinRM) debe registrar obligatoriamente en un log estructurado (JSON) el timestamp, el usuario que autorizó la ejecución, el comando exacto enviado y el código de salida devuelto por el sistema operativo.