# KernelIA - Plan de Implementacion por Fases del RAG Tecnico del Nucleo

Fecha: 2026-07-12

Estado: plan de implementacion

Documento base relacionado:

- [KERNELIA_RAG_TECNICO_NUCLEO_WINDOWS_2026-07-12.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_RAG_TECNICO_NUCLEO_WINDOWS_2026-07-12.md)

---

## 1. Objetivo

Implementar en KernelIA un RAG tecnico-operacional real, orientado a Windows, capaz de:

- comprender mejor la consulta del usuario;
- recuperar conocimiento tecnico correcto;
- decidir con precision entre aclarar, explicar, simular, ejecutar o escalar;
- reducir alucinaciones;
- impedir decisiones operativas erroneas;
- ejecutar herramientas solo bajo evidencia, policy y verificacion.

Este plan esta pensado para integrarse al backend actual `Tauri + Rust` sin romper el flujo de chat, tools, RBAC, auditoria ni core operativo ya existente.

---

## 2. Criterios de exito global

El proyecto se considerara bien implementado cuando KernelIA logre:

- enrutar consultas a una especialidad Windows correcta;
- recuperar contexto tecnico util y auditable;
- distinguir entre conocimiento estatico y estado vivo del equipo;
- no ejecutar acciones mutantes sin policy ni evidencia;
- verificar post-ejecucion si la accion realmente resolvio;
- mantener memoria operacional por sesion;
- emitir trazas reproducibles por consulta.

---

## 3. Orden de implementacion recomendado

No empezar por embeddings.

El orden correcto es:

1. modelo de datos;
2. catalogo de tools y comandos;
3. corpus tecnico curado;
4. retrieval lexical y estructurado;
5. motor de decision;
6. estado vivo del endpoint;
7. memoria operacional;
8. trazabilidad avanzada;
9. retrieval semantico;
10. hardening y QA.

---

## 4. Fase 0 - Preparacion de arquitectura

## Objetivo

Preparar el repo para introducir la nueva capa RAG sin mezclarla desordenadamente con `router.rs` y `function_calling.rs`.

## Trabajo

- crear arbol modular para `src-tauri/src/rag/`;
- definir modulos nuevos en `src-tauri/src/ai/`;
- definir interfaces de alto nivel entre chat, rag, tools y core;
- decidir si la primera implementacion sera con SQLite embebido;
- documentar convenciones de nombres y ownership de modulos.

## Estructura sugerida

```text
src-tauri/src/rag/
  mod.rs
  models/
  storage/
  ingest/
  policies/
  retrieval/
  decision/
  memory/
  trace/

src-tauri/src/ai/
  query_analyzer.rs
  specialty_router.rs
  knowledge_retriever.rs
  command_retriever.rs
  live_state_retriever.rs
  confidence_engine.rs
  decision_engine.rs
  prompt_context_builder.rs
  memory_engine.rs
  trace_engine.rs
```

## Entregables

- estructura de carpetas;
- `mod.rs` base;
- contratos Rust iniciales;
- documento corto de convenciones.

## Definition of Done

- el proyecto compila con los modulos vacios conectados;
- existe el esqueleto oficial del subsistema RAG.

---

## 5. Fase 1 - Modelo de datos y almacenamiento local

## Objetivo

Construir la base persistente del RAG tecnico de KernelIA.

## Decisiones recomendadas

- iniciar con SQLite local embebido;
- usar FTS5 para retrieval lexical;
- dejar preparado el modelo para embeddings posteriores;
- persistir todo dentro de una carpeta de datos controlada por KernelIA.

## Bases o dominios logicos a implementar

- `rag_knowledge`
- `rag_commands`
- `rag_decision`
- `runtime_memory`
- `runtime_trace`
- `endpoint_snapshot`

## Tablas minimas de la primera iteracion

- `knowledge_document`
- `knowledge_chunk`
- `windows_command`
- `windows_command_alias`
- `tool_capability`
- `tool_command_binding`
- `query_category`
- `domain_specialty`
- `decision_policy`
- `confidence_policy`
- `risk_policy`
- `conversation_session`
- `conversation_message`
- `memory_snapshot`
- `memory_fact`
- `trace_request`
- `trace_retrieval_hit`
- `trace_decision`
- `trace_tool_call`
- `endpoint_snapshot`

## Trabajo

- definir modelos Rust `serde`;
- crear capa `storage/`;
- crear migraciones iniciales;
- definir seeds basicos para categorias y especialidades;
- definir repositorios de lectura/escritura.

## Entregables

- esquema SQLite inicial;
- migraciones versionadas;
- seeds iniciales;
- repositorios Rust.

## Definition of Done

- KernelIA puede crear y abrir la base local;
- las tablas existen con versionado;
- los seeds base cargan correctamente.

---

## 6. Fase 2 - Catalogo formal de tools y comandos Windows

## Objetivo

Convertir el conocimiento operativo actual de KernelIA en un catalogo formal, tipado y consultable.

## Alcance

Se debe cargar:

- tools actuales del backend;
- wrappers y alias;
- comandos PowerShell/CMD asociados;
- area de Windows;
- riesgo;
- policy;
- verificaciones;
- rollback posible.

## Fuentes del repo

- [KERNELIA_CATALOGO_BASE_TOOLS.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_CATALOGO_BASE_TOOLS.md)
- [KERNELIA_AGENTES_ESPECIALIZADOS.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_AGENTES_ESPECIALIZADOS.md)
- [src-tauri/src/tools/mod.rs](G:\DESARROLLOS\kernelia\src-tauri\src\tools\mod.rs)
- [src-tauri/src/tools/catalog_tools.rs](G:\DESARROLLOS\kernelia\src-tauri\src\tools\catalog_tools.rs)

## Trabajo

- extraer listado canonico de tools;
- asociar cada tool a una especialidad;
- definir alias naturales y tecnicos;
- registrar comandos subyacentes;
- registrar precondiciones y verificaciones;
- marcar tools de solo lectura, mutantes y criticas.

## Entregables

- seed del catalogo completo;
- cargador de catalogo;
- validacion de consistencia entre tools del codigo y tools de la base.

## Definition of Done

- toda tool ejecutable del backend tiene registro formal en base;
- toda tool tiene especialidad, riesgo y policy minima;
- el sistema puede buscar una tool por nombre, alias o dominio.

---

## 7. Fase 3 - Corpus tecnico curado por especialidad

## Objetivo

Construir el conocimiento tecnico reutilizable que el agente consultara antes de decidir.

## Especialidades iniciales obligatorias

- network
- processes
- services
- maintenance
- security
- drivers
- filesystem
- performance
- software

## Tipos de documento a cargar

- `overview`
- `playbook`
- `faq`
- `symptom_matrix`
- `root_cause`
- `tool_usage`
- `guardrail`
- `verification`
- `rollback`
- `known_issue`

## Trabajo

- definir plantilla editorial de documentos;
- fragmentar conocimiento en chunks cortos;
- anclar cada chunk a su especialidad;
- definir `entity_key` por componente relevante;
- cargar conocimiento inicial curado.

## Regla editorial

Cada documento debe responder al menos una de estas preguntas:

- que es;
- cuando aplica;
- cuando no aplica;
- como se verifica;
- como se remedia;
- como se revierte;
- cuando escalar.

## Entregables

- carpeta fuente del corpus;
- proceso de ingesta;
- corpus inicial por especialidad.

## Definition of Done

- cada especialidad inicial tiene documentos utiles;
- el corpus no depende de prompts libres;
- el sistema puede recuperar chunks por keyword y por especialidad.

---

## 8. Fase 4 - Query analyzer y specialty router

## Objetivo

Reemplazar las heuristicas simples actuales por un analizador de consulta mas robusto y orientado a dominio tecnico.

## Trabajo

- evolucionar `intent_engine` a un `query_analyzer`;
- detectar:
  - query category,
  - specialty,
  - sintomas,
  - componente afectado,
  - urgencia,
  - riesgo preliminar,
  - necesidad de aclaracion;
- construir `QueryAnalysis`.

## Contrato recomendado

```rust
pub struct QueryAnalysis {
    pub normalized_text: String,
    pub query_category: String,
    pub specialty: String,
    pub urgency: String,
    pub symptoms: Vec<String>,
    pub entities: Vec<String>,
    pub ambiguity_score: f32,
    pub requires_clarification: bool,
}
```

## Integracion

- `router.rs` deja de depender solo de shortcuts textuales;
- el chat usa primero `query_analyzer`;
- `specialty_router` define el dominio candidato principal.

## Entregables

- `query_analyzer.rs`
- `specialty_router.rs`
- tests unitarios por dominio.

## Definition of Done

- consultas como `dns`, `spooler`, `codigo 43`, `disco al 100` enrutan correctamente;
- las queries ambiguas producen bandera de aclaracion.

---

## 9. Fase 5 - Retrieval lexical y estructurado

## Objetivo

Tener un RAG util antes de meter embeddings.

## Trabajo

- retrieval lexical sobre:
  - `knowledge_chunk`
  - `windows_command_alias`
  - `tool_capability`
  - `symptom_taxonomy`
- retrieval estructurado sobre:
  - politicas
  - riesgo
  - decision
  - especialidad
- ranking inicial por score lexical + bonuses.

## Fuente de verdad

Para KernelIA, en esta fase la prioridad debe ser:

1. tool/policy exacta;
2. comando Windows exacto;
3. sintoma exacto;
4. chunk tecnico de misma especialidad;
5. memoria de sesion.

## Entregables

- `knowledge_retriever.rs`
- `command_retriever.rs`
- `retrieval_result` estructurado;
- tests de ranking.

## Definition of Done

- el sistema recupera contexto util por especialidad;
- las coincidencias exactas ganan a la similitud vaga;
- el retrieval devuelve evidencia, no solo texto.

---

## 10. Fase 6 - Confidence engine y decision engine

## Objetivo

Evitar alucinacion y malas decisiones.

## Trabajo

- implementar `confidence_engine`;
- calcular score con:
  - lexical score,
  - specialty match,
  - exact match bonus,
  - ambiguity penalty,
  - short query penalty,
  - policy conflict penalty;
- implementar `decision_engine`;
- producir `DecisionEnvelope`.

## Modos de salida obligatorios

- `clarify`
- `explain`
- `simulate`
- `execute`
- `escalate`
- `deny`

## Entregables

- `confidence_engine.rs`
- `decision_engine.rs`
- seeds de `decision_policy` y `confidence_policy`.

## Definition of Done

- consultas ambiguas ya no disparan respuestas falsas de confianza alta;
- acciones mutantes no pasan a ejecucion sin policy valida;
- el motor puede explicar por que decide aclarar o ejecutar.

---

## 11. Fase 7 - Integracion con estado vivo del endpoint

## Objetivo

Evitar que el RAG se apoye solo en conocimiento estatico.

## Trabajo

- normalizar snapshots de endpoint;
- crear `live_state_retriever`;
- consultar telemetria, procesos, red, servicios y seguridad segun especialidad;
- generar `LiveStateContext`.

## Regla critica

Si el estado vivo contradice el conocimiento recuperado, manda el estado vivo.

## Ejemplos

- si el usuario dice `sin internet` pero el diagnostico muestra conectividad normal, el sistema debe pedir aclaracion o explorar otra capa;
- si el servicio ya esta corriendo, no debe recomendar reiniciarlo sin razon.

## Entregables

- `live_state_retriever.rs`
- normalizador de snapshots;
- tablas `endpoint_*` o almacenamiento JSON estructurado.

## Definition of Done

- el motor puede enriquecer decisiones con estado actual del equipo;
- el contexto final diferencia claramente conocimiento y evidencia viva.

---

## 12. Fase 8 - Prompt context builder y control de LLM

## Objetivo

Entregar al LLM solo el contexto correcto, sintetizado y gobernado.

## Trabajo

- construir `prompt_context_builder`;
- separar:
  - decision,
  - live state,
  - knowledge,
  - tools permitidas,
  - tools denegadas,
  - memoria corta;
- actualizar `router.rs` y `function_calling.rs` para consumir ese contexto;
- evitar prompts largos no estructurados.

## Entregables

- `prompt_context_builder.rs`
- nuevo formato de contexto inyectado;
- integracion con el loop de chat.

## Definition of Done

- el LLM recibe un contexto compacto y auditable;
- las respuestas siguen policy;
- baja el ruido contextual en modelos locales.

---

## 13. Fase 9 - Verificacion post-tool y remediacion segura

## Objetivo

Garantizar que ejecutar no sea equivalente a "dar por resuelto".

## Trabajo

- modelar `precondition`, `execute`, `verify`, `rollback`;
- asociar verificacion por tool;
- registrar resultado tecnico post-accion;
- actualizar memoria y trace con la verificacion.

## Ejemplos obligatorios

- `restart_service` -> `get_service_status`
- `flush_dns_cache` -> `dns_lookup`
- `repair_system_files` -> salida SFC + estado posterior
- `run_cleanup` -> comparar espacio antes/despues

## Entregables

- `tool_postcondition` y `tool_evidence_rule`;
- engine de verificacion;
- hooks post-ejecucion.

## Definition of Done

- KernelIA puede distinguir entre accion ejecutada y problema resuelto;
- toda accion mutante deja evidencia de verificacion.

---

## 14. Fase 10 - Memoria operacional por sesion

## Objetivo

Mantener continuidad tecnica real.

## Trabajo

- persistir sesiones y mensajes;
- construir snapshots tecnicos de memoria;
- guardar:
  - objetivo del usuario,
  - especialidad primaria,
  - sintomas,
  - acciones ejecutadas,
  - hallazgos,
  - preguntas abiertas,
  - estado de resolucion;
- leer memoria util en consultas posteriores.

## Entregables

- `memory_engine.rs`
- persistencia de `memory_snapshot` y `memory_fact`;
- reglas de merge de memoria.

## Definition of Done

- KernelIA recuerda contexto tecnico relevante dentro de la sesion;
- evita repetir tools ya ejecutadas sin razon;
- usa memoria sin sobreescribir evidencia actual.

---

## 15. Fase 11 - Trace engine, auditoria y explicabilidad

## Objetivo

Hacer reproducible cada decision del agente.

## Trabajo

- crear `trace_engine`;
- registrar:
  - query original,
  - query normalizada,
  - categoria,
  - especialidad,
  - hits recuperados,
  - score de confianza,
  - decision,
  - tools,
  - verificacion,
  - errores;
- conectar con auditoria actual.

## Entregables

- `trace_engine.rs`
- tablas `trace_*`
- export simple para QA.

## Definition of Done

- cada respuesta importante puede explicarse hacia atras;
- QA puede reconstruir por que KernelIA decidio lo que decidio.

---

## 16. Fase 12 - Retrieval semantico y ranking hibrido

## Objetivo

Agregar embeddings solo cuando el motor base ya es confiable.

## Trabajo

- generar embeddings por chunk;
- almacenar embeddings;
- combinar semantic + lexical + policy bonuses;
- aplicar cohesion por especialidad y entidad dominante.

## Requisito previo

No iniciar esta fase hasta tener retrieval lexical y decision engine estables.

## Entregables

- `knowledge_chunk_embedding`
- proceso de generacion de embeddings;
- ranking hibrido.

## Definition of Done

- mejora recall semantico sin degradar precision operacional;
- queries abiertas encuentran mejor conocimiento, sin romper queries exactas.

---

## 17. Fase 13 - QA tecnico y hardening

## Objetivo

Asegurar que el agente sea explotable operativamente.

## Trabajo

- tests unitarios por modulo;
- tests de retrieval;
- tests de decision;
- tests de no-alucinacion;
- tests de denial seguro;
- tests de conflicto entre memoria y live state;
- tests de verificacion post-tool;
- smoke tests por especialidad;
- medicion de latencias.

## Casos obligatorios

- consulta corta ambigua;
- comando sensible solicitado por usuario Viewer;
- retrieval con chunks conflictivos;
- accion R3 sin snapshot;
- respuesta con confidence baja;
- servicio reiniciado pero no recuperado;
- memoria vieja contradiciendo snapshot nuevo.

## Entregables

- suite de tests;
- dataset de pruebas;
- reporte de cobertura funcional del RAG.

## Definition of Done

- el subsistema demuestra comportamiento estable ante casos normales y peligrosos;
- existen pruebas para no regresionar.

---

## 18. Fase 14 - Exposicion gradual en UI

## Objetivo

Introducir el nuevo nucleo sin romper la experiencia actual.

## Trabajo

- feature flag del RAG nuevo;
- modo comparativo entre flujo actual y flujo nuevo;
- panel interno de decision/trace para QA;
- salida visible de nivel de confianza y especialidad cuando convenga.

## Entregables

- bandera `rag_engine_enabled`;
- panel de debug interno;
- rollout gradual.

## Definition of Done

- el RAG puede activarse por entorno o configuracion;
- QA puede comparar facilmente resultados.

---

## 19. Dependencias tecnicas sugeridas

Para primera version:

- SQLite
- FTS5
- `serde`
- `serde_json`
- `chrono`
- `uuid`

Para version hibrida posterior:

- motor de embeddings local o proveedor controlado;
- almacenamiento vectorial local o `pgvector` si se migra a PostgreSQL.

No introducir dependencias cloud obligatorias en la primera iteracion.

---

## 20. Riesgos principales del proyecto

- cargar demasiada complejidad antes de tener retrieval exacto;
- meter embeddings demasiado pronto;
- no curar bien el corpus tecnico;
- dejar que el LLM salte el decision engine;
- no modelar verificacion post-tool;
- mezclar memoria conversacional con estado vivo;
- no formalizar riesgos y especialidades;
- no sincronizar catalogo de tools con el codigo real.

---

## 21. Hitos recomendados

## Hito 1

Base local + catalogo + corpus inicial.

## Hito 2

Query analyzer + retrieval lexical + decision engine.

## Hito 3

Integracion con estado vivo + verificacion post-tool.

## Hito 4

Memoria operacional + trazabilidad total.

## Hito 5

Ranking hibrido semantico + QA completo.

---

## 22. Secuencia de ejecucion recomendada en el repo

1. crear modulos vacios y storage;
2. crear migraciones y seeds;
3. volcar tools actuales;
4. curar corpus por especialidad;
5. implementar analyzer y router;
6. implementar retrieval lexical;
7. implementar decision engine;
8. integrar con `router.rs`;
9. integrar con `function_calling.rs`;
10. agregar live state;
11. agregar memoria;
12. agregar trace;
13. endurecer con tests;
14. agregar embeddings.

---

## 23. Dictamen final

La implementacion correcta del nuevo RAG de KernelIA no es una fase unica.

Debe construirse como una evolucion controlada del nucleo actual:

- primero estructura;
- luego catalogo y corpus;
- despues retrieval y decision;
- despues live state y verificacion;
- despues memoria y trazabilidad;
- recien despues semantica avanzada.

Ese orden minimiza riesgo, mejora precision real y acerca a KernelIA a un agente tecnico verdaderamente inteligente y confiable.
