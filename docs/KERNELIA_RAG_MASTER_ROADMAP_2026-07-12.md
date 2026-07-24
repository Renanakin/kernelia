# KernelIA - Master Roadmap del RAG Tecnico del Nucleo

Fecha: 2026-07-12

Estado: roadmap maestro

Documentos base:

- [KERNELIA_RAG_TECNICO_NUCLEO_WINDOWS_2026-07-12.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_RAG_TECNICO_NUCLEO_WINDOWS_2026-07-12.md)
- [KERNELIA_RAG_PLAN_IMPLEMENTACION_POR_FASES_2026-07-12.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_RAG_PLAN_IMPLEMENTACION_POR_FASES_2026-07-12.md)
- [KERNELIA_RAG_INFORME_BACKLOG_Y_SKILLS_2026-07-12.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_RAG_INFORME_BACKLOG_Y_SKILLS_2026-07-12.md)

---

## 1. Proposito

Este roadmap maestro unifica en una sola vista:

- la vision del nuevo RAG tecnico de KernelIA;
- la arquitectura objetivo;
- el backlog por fases;
- las skills recomendadas;
- los entregables;
- los gates de revision;
- el orden real de ejecucion.

La meta no es agregar un modulo RAG aislado.
La meta es transformar KernelIA en un motor de decision tecnica, con contexto confiable, memoria operacional, control de riesgo y verificacion real sobre Windows.

---

## 2. Vision objetivo

KernelIA debe evolucionar desde:

- chat + tools + heuristicas

hacia:

- analisis de consulta
- routing por especialidad Windows
- retrieval hibrido
- decision engine
- contexto vivo del endpoint
- tool calling gobernado
- verificacion post-tool
- memoria operacional
- trazabilidad completa

El resultado esperado es un agente que:

- entienda mejor la pregunta del usuario;
- no alucine;
- no recomiende acciones incorrectas;
- no ejecute fuera de policy;
- recuerde el contexto tecnico de la sesion;
- valide si lo que hizo realmente resolvio el problema.

---

## 3. Estado actual del proyecto

## Lo que KernelIA ya tiene

- backend Tauri + Rust robusto
- tools reales de sistema
- RBAC y MegaBoss
- auditoria y snapshots
- chat con function calling
- intent engine tecnico inicial
- quick checks y modulos operativos

## Lo que KernelIA aun no tiene

- base RAG persistente
- corpus tecnico curado por especialidad
- catalogo formal de comandos y tools
- retrieval lexical/semantico desacoplado
- confidence engine real
- decision engine formal
- memoria operacional estructurada
- verificacion estandar post-tool

---

## 4. Arquitectura objetivo resumida

```text
Usuario
  -> Ingreso seguro
  -> Query Analyzer
  -> Specialty Router
  -> Retrieval Hibrido
       -> Knowledge
       -> Commands/Tools
       -> Policies
       -> Session Memory
       -> Live Endpoint State
  -> Confidence Engine
  -> Decision Engine
       -> clarify
       -> explain
       -> simulate
       -> execute
       -> escalate
       -> deny
  -> Prompt Context Builder
  -> LLM controlado
  -> Tool calling tipado
  -> Verificacion
  -> Memoria + Trace + Auditoria
```

---

## 5. Pilares del roadmap

## Pilar 1 - Conocimiento tecnico curado

Base documental estructurada por especialidad Windows.

## Pilar 2 - Catalogo operativo formal

Tools, comandos, riesgo, permisos, precondiciones y verificaciones.

## Pilar 3 - Toma de decision gobernada

Confidence gating, policy, riesgo y routing tecnico.

## Pilar 4 - Evidencia viva del endpoint

El estado actual del equipo debe tener prioridad sobre inferencia vaga.

## Pilar 5 - Memoria operacional

Persistencia de hechos tecnicos utiles, no solo transcript.

## Pilar 6 - Seguridad y trazabilidad

Nada sensible sin policy, evidencia y auditabilidad.

---

## 6. Mapa de datos del RAG

KernelIA debe organizar su nueva capa en seis dominios logicos:

1. `rag_knowledge`
2. `rag_commands`
3. `rag_decision`
4. `runtime_memory`
5. `runtime_trace`
6. `endpoint_snapshot`

Resumen de contenido:

- `rag_knowledge`: playbooks, FAQs, matrices de sintomas, remediaciones, guardrails
- `rag_commands`: comandos Windows, tools, aliases, bindings, preconditions, postconditions
- `rag_decision`: categorias, especialidades, policies, riesgo, escalamiento, thresholds
- `runtime_memory`: sesiones, hechos, snapshots tecnicos, acciones previas
- `runtime_trace`: retrieval, score, decision, tool calls, errores, latencias
- `endpoint_snapshot`: estado vivo del equipo

---

## 7. Especialidades Windows objetivo

Especialidades iniciales del motor:

1. `system`
2. `telemetry`
3. `network`
4. `processes`
5. `services`
6. `maintenance`
7. `security`
8. `drivers`
9. `filesystem`
10. `audit`
11. `performance`
12. `software`
13. `sensitive_ops`
14. `megaboss`

Cada especialidad debe tener:

- corpus tecnico
- tools asociadas
- comandos Windows asociados
- sintomas frecuentes
- causas probables
- verificaciones
- reglas de escalamiento

---

## 8. Fases maestras

## Fase 0 - Preparacion de arquitectura

Objetivo:

- crear el boundary formal del subsistema RAG.

Entregables:

- estructura modular
- ADR de arquitectura
- feature flag inicial

Skills clave:

- `context-map`
- `architecture-blueprint-generator`
- `create-architectural-decision-record`

## Fase 1 - Modelo de datos y storage

Objetivo:

- construir la base persistente del RAG tecnico.

Entregables:

- esquema SQLite inicial
- migraciones
- seeds
- repositorios

Skills clave:

- `create-specification`
- `breakdown-feature-implementation`
- `documentation-writer`

## Fase 2 - Catalogo formal de tools y comandos

Objetivo:

- convertir el runtime actual de tools en conocimiento formal consultable.

Entregables:

- inventario de tools
- catalogo de comandos
- guardrails por tool

Skills clave:

- `context-map`
- `security-best-practices`
- `create-specification`

## Fase 3 - Corpus tecnico curado

Objetivo:

- crear el conocimiento tecnico que el agente consultara.

Entregables:

- corpus por especialidad
- pipeline de ingesta
- chunks persistidos

Skills clave:

- `documentation-writer`
- `doc`
- `create-readme`

## Fase 4 - Query analyzer y specialty router

Objetivo:

- mejorar comprension y ruteo de consultas.

Entregables:

- `query_analyzer.rs`
- `specialty_router.rs`
- tests de clasificacion

Skills clave:

- `breakdown-feature-implementation`
- `refactor-plan`
- `review-and-refactor`

## Fase 5 - Retrieval lexical y estructurado

Objetivo:

- tener RAG util sin depender aun de embeddings.

Entregables:

- retrievers base
- ranking lexical
- tests de precision

Skills clave:

- `architecture-blueprint-generator`
- `breakdown-test`
- `review-and-refactor`

## Fase 6 - Confidence engine y decision engine

Objetivo:

- impedir respuestas y acciones mal fundadas.

Entregables:

- `confidence_engine.rs`
- `decision_engine.rs`
- policies cargadas

Skills clave:

- `structured-autonomy-plan`
- `security-review`
- `breakdown-test`

## Fase 7 - Live state retriever

Objetivo:

- cruzar conocimiento con evidencia viva del endpoint.

Entregables:

- normalizador de snapshots
- `live_state_retriever.rs`
- reglas de conflicto

Skills clave:

- `context-map`
- `security-best-practices`
- `breakdown-feature-implementation`

## Fase 8 - Prompt context builder e integracion LLM

Objetivo:

- inyectar al modelo solo contexto gobernado.

Entregables:

- `prompt_context_builder.rs`
- integracion con `router.rs`
- integracion con `function_calling.rs`

Skills clave:

- `refactor-plan`
- `review-and-refactor`
- `breakdown-test`

## Fase 9 - Verificacion post-tool

Objetivo:

- cerrar el ciclo tecnico de ejecucion.

Entregables:

- reglas de verificacion
- hooks post-tool
- trazas de verificacion

Skills clave:

- `structured-autonomy-implement`
- `security-review`
- `breakdown-test`

## Fase 10 - Memoria operacional

Objetivo:

- mantener continuidad tecnica real de la sesion.

Entregables:

- `memory_engine.rs`
- snapshots tecnicos
- politicas de merge

Skills clave:

- `architecture-blueprint-generator`
- `create-specification`
- `documentation-writer`

## Fase 11 - Trace y explicabilidad

Objetivo:

- hacer auditable toda decision del motor.

Entregables:

- `trace_engine.rs`
- tablas `trace_*`
- export de QA

Skills clave:

- `documentation-writer`
- `codex-security:validation`
- `security-best-practices`

## Fase 12 - Retrieval semantico e hibrido

Objetivo:

- mejorar recall sin perder precision.

Entregables:

- embeddings por chunk
- ranking hibrido
- calibration

Skills clave:

- `architecture-blueprint-generator`
- `breakdown-feature-implementation`
- `breakdown-test`

## Fase 13 - QA y hardening

Objetivo:

- asegurar estabilidad, precision y seguridad.

Entregables:

- suite de pruebas
- checklist de seguridad
- reporte de validacion

Skills clave:

- `polyglot-test-agent`
- `security-review`
- `codex-security:security-scan`
- `breakdown-test`

## Fase 14 - Rollout y QA UI

Objetivo:

- activar el nuevo motor sin romper la operacion actual.

Entregables:

- feature flag operativa
- panel interno de QA
- manual interno

Skills clave:

- `breakdown-feature-implementation`
- `documentation-writer`
- `review-and-refactor`

---

## 9. Orden real de ejecucion

Orden recomendado:

1. Fase 0
2. Fase 1
3. Fase 2
4. Fase 3
5. Fase 4
6. Fase 5
7. Fase 6
8. Gate tecnico 1
9. Fase 7
10. Fase 8
11. Fase 9
12. Fase 10
13. Fase 11
14. Gate tecnico 2
15. Fase 12
16. Fase 13
17. Fase 14

No conviene empezar por embeddings ni por UI.

---

## 10. Backlog ejecutivo resumido

## Bloque A - Fundaciones

- definir arquitectura
- crear storage
- crear modelos de datos
- crear seeds

## Bloque B - Conocimiento

- catalogar tools y comandos
- curar corpus tecnico
- crear pipeline de ingesta

## Bloque C - Inteligencia de decision

- query analyzer
- specialty router
- retrieval lexical
- confidence engine
- decision engine

## Bloque D - Ejecucion segura

- live state retriever
- prompt context builder
- post-tool verification
- integration with tool runtime

## Bloque E - Continuidad y observabilidad

- memoria operacional
- trace engine
- audit enrichment

## Bloque F - Maduracion

- retrieval semantico
- QA completo
- hardening
- rollout gradual

---

## 11. Skills prioritarias para el proyecto

Si hubiese que priorizar solo las mas importantes para este roadmap:

1. `context-map`
2. `architecture-blueprint-generator`
3. `create-implementation-plan`
4. `breakdown-feature-implementation`
5. `create-specification`
6. `documentation-writer`
7. `security-best-practices`
8. `structured-autonomy-plan`
9. `breakdown-test`
10. `security-review`
11. `review-and-refactor`
12. `codex-security:validation`

---

## 12. Gates de revision

## Gate 1 - Arquitectura y datos

Validar:

- boundaries correctos
- schema coherente
- storage versionado
- seeds base correctos

## Gate 2 - Precision de decision

Validar:

- specialty routing
- retrieval exacto
- confidence thresholds
- decision modes

## Gate 3 - Seguridad operativa

Validar:

- policies por riesgo
- bloqueo de tools sensibles
- snapshot requirements
- verificacion post-tool

## Gate 4 - Continuidad y QA

Validar:

- memoria operacional
- explicabilidad
- trazabilidad
- cobertura de pruebas

---

## 13. Riesgos principales

- introducir demasiada complejidad sin boundaries claros
- meter embeddings demasiado pronto
- no curar bien el corpus tecnico
- dejar decisiones al LLM en vez de al decision engine
- no modelar verificacion post-tool
- permitir conflicto entre memoria, corpus y estado vivo
- no formalizar bien las especialidades
- no mantener sincronizado el catalogo de tools con el codigo

---

## 14. Criterios de exito final

El roadmap se considerara exitoso si al final KernelIA puede:

- detectar mejor que quiso decir el usuario;
- enrutar por especialidad correcta;
- responder con conocimiento tecnico auditable;
- decidir si aclarar, explicar, simular o ejecutar;
- bloquear acciones fuera de policy;
- verificar el resultado de la accion;
- recordar contexto tecnico de la sesion;
- emitir una traza completa y comprensible por QA.

---

## 15. Recomendacion final

La mejor estrategia para KernelIA es construir este RAG como una evolucion del nucleo, no como un anexo.

Primero:

- estructura
- catalogo
- corpus
- decision

Despues:

- live state
- verificacion
- memoria
- trace

Y solo al final:

- embeddings
- optimizacion semantica
- rollout total

Ese orden mantiene el control tecnico, reduce riesgo y maximiza la calidad del agente.
