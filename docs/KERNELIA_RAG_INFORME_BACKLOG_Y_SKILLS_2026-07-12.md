# KernelIA - Informe de Revision, Backlog Completo y Skills por Fase para el RAG Tecnico

Fecha: 2026-07-12

Estado: informe para revision tecnica

Documentos relacionados:

- [KERNELIA_RAG_TECNICO_NUCLEO_WINDOWS_2026-07-12.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_RAG_TECNICO_NUCLEO_WINDOWS_2026-07-12.md)
- [KERNELIA_RAG_PLAN_IMPLEMENTACION_POR_FASES_2026-07-12.md](G:\DESARROLLOS\kernelia\docs\KERNELIA_RAG_PLAN_IMPLEMENTACION_POR_FASES_2026-07-12.md)

---

## 1. Objetivo del informe

Este informe consolida:

- el backlog completo del RAG tecnico de KernelIA;
- las skills recomendadas para ejecutar cada fase con alto estandar;
- los criterios de revision para asegurar una implementacion consistente;
- el orden operativo sugerido para construir el nucleo sin degradar el sistema actual.

La meta no es solo implementar un RAG.
La meta es construir un motor de decision tecnico, confiable, trazable y operativo sobre Windows.

---

## 2. Dictamen ejecutivo

KernelIA ya tiene:

- un backend robusto de tools;
- RBAC y auditoria;
- chat con function calling;
- analisis de intencion tecnico inicial;
- snapshots y operaciones de core.

KernelIA aun no tiene:

- RAG tecnico persistente;
- catalogo tipado de conocimiento y comandos;
- retrieval hibrido formal;
- confidence gating real;
- decision engine desacoplado del LLM;
- memoria operacional estructurada;
- verificacion sistematica post-tool.

Por eso el trabajo correcto no es ajustar prompts.
El trabajo correcto es construir una nueva capa de razonamiento.

---

## 3. Enfoque de implementacion recomendado

La implementacion debe hacerse con estos principios:

- primero estructura, despues inteligencia;
- primero precision lexical y de policy, despues semantica;
- primero decision controlada, despues autonomia;
- primero evidencia y verificacion, despues ejecucion mas agresiva;
- primero skills de arquitectura y modelado, despues skills de optimizacion.

---

## 4. Skills disponibles mas relevantes para este proyecto

Estas son las skills mas utiles del entorno actual para una implementacion de alto nivel en este repo:

- `architecture-blueprint-generator`
- `architecture-blueprint-generator`
- `create-implementation-plan`
- `breakdown-plan`
- `breakdown-feature-implementation`
- `breakdown-test`
- `context-map`
- `create-specification`
- `create-architectural-decision-record`
- `documentation-writer`
- `doc`
- `refactor-plan`
- `review-and-refactor`
- `rust-mcp-server-generator`
- `security-best-practices`
- `security-review`
- `codex-security:security-scan`
- `codex-security:validation`
- `structured-autonomy-plan`
- `structured-autonomy-implement`
- `structured-autonomy-generate`
- `pytest-coverage`
- `polyglot-test-agent`
- `make-repo-contribution`
- `create-readme`

Skills del ecosistema Rust o arquitectura que tienen mas valor practico aqui:

- `architecture-blueprint-generator`
- `context-map`
- `create-implementation-plan`
- `breakdown-feature-implementation`
- `breakdown-test`
- `refactor-plan`
- `security-best-practices`
- `security-review`
- `documentation-writer`

Nota:

No todas las skills escriben codigo directamente.
Algunas son especialmente utiles para:

- definir alcance;
- endurecer arquitectura;
- diseñar pruebas;
- revisar seguridad;
- ordenar backlog.

---

## 5. Matriz general de fases, objetivos y skills

| Fase | Objetivo | Skills principales |
|---|---|---|
| 0 | Preparar arquitectura | `architecture-blueprint-generator`, `context-map`, `create-architectural-decision-record` |
| 1 | Modelo de datos y storage | `create-specification`, `breakdown-feature-implementation`, `documentation-writer` |
| 2 | Catalogo de tools y comandos | `context-map`, `create-specification`, `security-best-practices` |
| 3 | Corpus tecnico curado | `documentation-writer`, `doc`, `create-specification` |
| 4 | Query analyzer y specialty router | `breakdown-feature-implementation`, `refactor-plan`, `review-and-refactor` |
| 5 | Retrieval lexical y estructurado | `architecture-blueprint-generator`, `breakdown-feature-implementation`, `breakdown-test` |
| 6 | Confidence y decision engine | `structured-autonomy-plan`, `security-review`, `breakdown-test` |
| 7 | Live state retriever | `context-map`, `breakdown-feature-implementation`, `security-best-practices` |
| 8 | Prompt context builder e integracion LLM | `refactor-plan`, `review-and-refactor`, `breakdown-test` |
| 9 | Verificacion post-tool | `structured-autonomy-implement`, `security-review`, `breakdown-test` |
| 10 | Memoria operacional | `architecture-blueprint-generator`, `create-specification`, `documentation-writer` |
| 11 | Trace y explicabilidad | `documentation-writer`, `security-best-practices`, `codex-security:validation` |
| 12 | Retrieval semantico hibrido | `architecture-blueprint-generator`, `breakdown-feature-implementation`, `breakdown-test` |
| 13 | QA y hardening | `breakdown-test`, `polyglot-test-agent`, `security-review`, `codex-security:security-scan` |
| 14 | Rollout y UI interna de QA | `breakdown-feature-implementation`, `documentation-writer`, `review-and-refactor` |

---

## 6. Backlog maestro por fases

## Fase 0 - Preparacion de arquitectura

## Objetivo

Separar formalmente el subsistema RAG del flujo actual de chat y tools.

## Skills recomendadas

- `architecture-blueprint-generator`
- `context-map`
- `create-architectural-decision-record`
- `create-implementation-plan`

## Backlog

### Epic 0.1 - Definir arquitectura del subsistema RAG

- mapear dependencias entre `ai/`, `tools/`, `core/` y `config/`
- definir boundaries entre chat orchestration y decision engine
- definir contratos entre `router`, `rag`, `tools` y `audit`
- emitir ADR de arquitectura

### Epic 0.2 - Crear esqueleto de modulos

- crear `src-tauri/src/rag/`
- crear modulos base de `models`, `storage`, `retrieval`, `decision`, `memory`, `trace`
- registrar nuevos `mod.rs`

### Epic 0.3 - Preparar feature flag

- definir `rag_engine_enabled`
- permitir convivencia entre flujo actual y flujo nuevo

## Entregables

- mapa de arquitectura
- ADR inicial
- estructura modular base

## Revision

- el subsistema queda desacoplado
- no se introduce logica del RAG directamente dentro de componentes viejos sin boundary claro

---

## Fase 1 - Modelo de datos y almacenamiento

## Objetivo

Crear el mapa real de bases de datos y el storage local inicial.

## Skills recomendadas

- `create-specification`
- `breakdown-feature-implementation`
- `documentation-writer`
- `refactor-plan`

## Backlog

### Epic 1.1 - Disenar esquema de datos

- definir entidades para knowledge
- definir entidades para commands/tools
- definir entidades para decision/policy
- definir entidades para memory
- definir entidades para trace
- definir entidades para snapshot endpoint

### Epic 1.2 - Elegir almacenamiento inicial

- confirmar SQLite como opcion Fase 1
- definir ubicacion del archivo
- definir estrategia de migraciones

### Epic 1.3 - Implementar capa `storage`

- crear repositorios
- crear migraciones
- crear seeds base

### Epic 1.4 - Normalizar modelos Rust

- structs para tablas base
- DTOs para retrieval y decision

## Entregables

- esquema inicial
- migraciones
- repositorios
- seeds

## Revision

- el storage soporta evolucion
- las tablas no mezclan transcript con memoria tecnica

---

## Fase 2 - Catalogo formal de tools y comandos Windows

## Objetivo

Formalizar la semantica operativa de todo lo que KernelIA sabe ejecutar.

## Skills recomendadas

- `context-map`
- `create-specification`
- `security-best-practices`
- `documentation-writer`

## Backlog

### Epic 2.1 - Inventario tecnico del backend actual

- extraer todas las tools de `tools/mod.rs`
- extraer tools de `catalog_tools.rs`
- mapear aliases y permisos

### Epic 2.2 - Clasificacion por especialidad

- asignar cada tool a una especialidad Windows
- asignar nivel de riesgo
- asignar mutabilidad

### Epic 2.3 - Carga de comandos Windows

- modelar comandos PowerShell y CMD usados por tools
- relacionarlos con `tool_command_binding`
- capturar precondiciones y verificaciones

### Epic 2.4 - Guardrails de tool

- registrar `min_role`
- registrar `requires_snapshot`
- registrar `requires_megaboss`
- registrar `verification_required`

## Entregables

- catalogo completo de tools
- catalogo inicial de comandos Windows asociados
- reglas de guardrails cargadas

## Revision

- ninguna tool del runtime queda sin metadata
- ninguna accion sensible queda sin policy

---

## Fase 3 - Corpus tecnico curado por especialidad

## Objetivo

Construir la base de conocimiento real del RAG.

## Skills recomendadas

- `documentation-writer`
- `doc`
- `create-specification`
- `create-readme`

## Backlog

### Epic 3.1 - Definir formato editorial del corpus

- plantilla por `playbook`
- plantilla por `faq`
- plantilla por `symptom_matrix`
- plantilla por `guardrail`

### Epic 3.2 - Curar corpus por especialidad

- network
- services
- performance
- security
- drivers
- filesystem
- maintenance
- software

### Epic 3.3 - Ingesta del corpus

- parser de markdown
- generacion de chunks
- asignacion de `entity_key`
- calculo de hash/version

## Entregables

- corpus inicial usable
- pipeline de ingesta
- chunks persistidos

## Revision

- el corpus es tecnico, no marketing
- cada especialidad tiene base suficiente para responder y decidir

---

## Fase 4 - Query analyzer y specialty router

## Objetivo

Mejorar la comprension de consulta antes del retrieval.

## Skills recomendadas

- `breakdown-feature-implementation`
- `refactor-plan`
- `review-and-refactor`
- `breakdown-test`

## Backlog

### Epic 4.1 - Diseno del `QueryAnalysis`

- definir categorias de query
- definir entidades tecnicas detectables
- definir sintomas y urgencia

### Epic 4.2 - Evolucion del intent engine

- extraer logica reusable
- separar analisis y ruteo
- introducir `specialty_router`

### Epic 4.3 - Clarification hints

- construir templates de aclaracion por dominio

## Entregables

- `query_analyzer.rs`
- `specialty_router.rs`
- tests unitarios

## Revision

- consultas cortas ambiguas ya no se responden con falsa seguridad

---

## Fase 5 - Retrieval lexical y estructurado

## Objetivo

Implementar retrieval exacto y confiable antes de embeddings.

## Skills recomendadas

- `architecture-blueprint-generator`
- `breakdown-feature-implementation`
- `breakdown-test`
- `review-and-refactor`

## Backlog

### Epic 5.1 - Retrieval sobre knowledge

- buscar por texto
- buscar por especialidad
- buscar por `entity_key`

### Epic 5.2 - Retrieval sobre tools/comandos

- buscar por alias natural
- buscar por nombre tecnico
- buscar por comando exacto

### Epic 5.3 - Retrieval sobre decision/policy

- cargar politicas aplicables a la consulta
- recuperar restricciones por rol y riesgo

### Epic 5.4 - Ranking inicial

- peso lexical
- bonus por exact match
- bonus por especialidad
- penalizacion por ambiguedad

## Entregables

- retrievers base
- ranking lexical
- pruebas de precision

## Revision

- `dns` no trae chunks de otras areas
- `spooler` prioriza servicios

---

## Fase 6 - Confidence engine y decision engine

## Objetivo

Controlar formalmente la toma de decision.

## Skills recomendadas

- `structured-autonomy-plan`
- `security-review`
- `breakdown-test`
- `create-specification`

## Backlog

### Epic 6.1 - Confidence model

- definir formula de score
- definir thresholds
- definir reasons codes

### Epic 6.2 - Decision envelope

- modelar `clarify`
- modelar `explain`
- modelar `simulate`
- modelar `execute`
- modelar `escalate`
- modelar `deny`

### Epic 6.3 - Politicas de ejecucion

- aplicar riesgo
- aplicar rol
- aplicar policy
- aplicar snapshot requirement

## Entregables

- `confidence_engine.rs`
- `decision_engine.rs`
- policies cargadas

## Revision

- el LLM ya no decide solo
- el motor puede bloquear acciones peligrosas antes del prompt

---

## Fase 7 - Live state retriever

## Objetivo

Cruzar conocimiento con evidencia viva del endpoint.

## Skills recomendadas

- `context-map`
- `breakdown-feature-implementation`
- `security-best-practices`
- `breakdown-test`

## Backlog

### Epic 7.1 - Snapshot normalizado del endpoint

- CPU
- RAM
- disco
- red
- procesos
- servicios
- drivers
- seguridad

### Epic 7.2 - Query contextual al estado vivo

- seleccionar solo lo relevante segun especialidad
- evitar sobrecargar contexto

### Epic 7.3 - Resolver conflictos entre corpus y estado vivo

- reglas de precedencia
- manejo de contradicciones

## Entregables

- `live_state_retriever.rs`
- snapshot normalizer
- tests de conflicto

## Revision

- el estado vivo prevalece sobre memoria o corpus si hay contradiccion

---

## Fase 8 - Prompt context builder e integracion LLM

## Objetivo

Entregar al modelo solo el contexto gobernado.

## Skills recomendadas

- `refactor-plan`
- `review-and-refactor`
- `breakdown-test`
- `documentation-writer`

## Backlog

### Epic 8.1 - Context builder

- decision
- live state
- knowledge top hits
- tools allowed
- tools denied
- memoria corta

### Epic 8.2 - Integracion con `router.rs`

- reemplazar enriquecimiento actual por contexto estructurado

### Epic 8.3 - Integracion con `function_calling.rs`

- solo ejecutar tools permitidas por `DecisionEnvelope`

## Entregables

- `prompt_context_builder.rs`
- integracion a chat pipeline

## Revision

- baja el ruido del prompt
- sube la consistencia de respuesta

---

## Fase 9 - Verificacion post-tool y remediacion segura

## Objetivo

Cerrar el ciclo tecnico de ejecucion.

## Skills recomendadas

- `structured-autonomy-implement`
- `security-review`
- `breakdown-test`
- `review-and-refactor`

## Backlog

### Epic 9.1 - Contrato por tool

- `precondition`
- `execute`
- `verify`
- `rollback`

### Epic 9.2 - Motor de verificacion

- verificar segun tipo de tool
- registrar evidencia post-tool

### Epic 9.3 - Resultado tecnico final

- distinguir `ejecutado`
- distinguir `verificado`
- distinguir `resuelto`

## Entregables

- reglas de verificacion
- hooks post-tool
- trazas de verificacion

## Revision

- una tool ya no se considera exito solo por no fallar

---

## Fase 10 - Memoria operacional

## Objetivo

Construir continuidad tecnica y no solo transcript.

## Skills recomendadas

- `architecture-blueprint-generator`
- `create-specification`
- `documentation-writer`
- `breakdown-test`

## Backlog

### Epic 10.1 - Modelo de memoria

- resumen tecnico
- hechos
- acciones
- estado de resolucion
- preguntas abiertas

### Epic 10.2 - Escritura de memoria

- construir snapshot despues de cada interaccion relevante

### Epic 10.3 - Lectura de memoria

- inyectar solo memoria util
- resolver expiracion y conflictos

## Entregables

- `memory_engine.rs`
- tablas de memoria
- politicas de merge

## Revision

- la memoria ayuda a resolver
- no reintroduce ruido ni contradicciones

---

## Fase 11 - Trace y explicabilidad

## Objetivo

Hacer auditable cada consulta del agente.

## Skills recomendadas

- `documentation-writer`
- `security-best-practices`
- `codex-security:validation`
- `breakdown-test`

## Backlog

### Epic 11.1 - Trace por request

- query original
- query normalizada
- categoria
- especialidad

### Epic 11.2 - Trace por retrieval

- hits
- scores
- source type

### Epic 11.3 - Trace por decision y tools

- score final
- modo de decision
- tools usadas
- verificacion
- errores

## Entregables

- `trace_engine.rs`
- export de trazas
- queries de QA

## Revision

- QA puede reconstruir por que KernelIA respondio y ejecuto lo que ejecuto

---

## Fase 12 - Retrieval semantico e hibrido

## Objetivo

Subir recall sin bajar precision.

## Skills recomendadas

- `architecture-blueprint-generator`
- `breakdown-feature-implementation`
- `breakdown-test`
- `review-and-refactor`

## Backlog

### Epic 12.1 - Embeddings por chunk

- definir proveedor o motor local
- generar embeddings
- persistir vectores

### Epic 12.2 - Ranking hibrido

- combinar lexical y vectorial
- cohesion por especialidad
- bonus por exact match y estado vivo

### Epic 12.3 - Calibration

- ajustar thresholds
- evitar degradar queries exactas

## Entregables

- pipeline de embeddings
- ranking hibrido
- pruebas comparativas

## Revision

- la semantica mejora queries abiertas
- no rompe consultas cortas tecnicas

---

## Fase 13 - QA, seguridad y hardening

## Objetivo

Dejar el motor listo para uso serio.

## Skills recomendadas

- `breakdown-test`
- `polyglot-test-agent`
- `security-review`
- `codex-security:security-scan`
- `codex-security:validation`

## Backlog

### Epic 13.1 - Test plan integral

- unitarios
- integracion
- regresion
- seguridad

### Epic 13.2 - Casos criticos

- consulta ambigua
- accion peligrosa
- conflicto entre dominios
- memoria vieja
- snapshot faltante

### Epic 13.3 - Security hardening

- validar egress
- endurecer acciones sensibles
- revisar retencion de trazas

## Entregables

- suite de pruebas
- checklist de seguridad
- reporte de validacion

## Revision

- el motor no se rompe en condiciones conflictivas
- la seguridad sigue gobernando la autonomia

---

## Fase 14 - Rollout, QA UI y adopcion controlada

## Objetivo

Activar el nuevo motor sin riesgos de despliegue.

## Skills recomendadas

- `breakdown-feature-implementation`
- `documentation-writer`
- `review-and-refactor`
- `create-readme`

## Backlog

### Epic 14.1 - Feature flag y modo comparativo

- activar/desactivar RAG nuevo
- comparar salida legacy vs nueva

### Epic 14.2 - Panel interno de QA

- decision mode
- confidence
- top hits
- tools allowed/denied
- reason codes

### Epic 14.3 - Manual operativo

- guia de uso para QA
- guia de tuning
- guia de revision de fallos

## Entregables

- rollout controlado
- panel de inspeccion
- manual interno

## Revision

- el cambio puede desplegarse de forma segura y auditable

---

## 7. Skills especificas por tipo de trabajo

## 7.1 Para arquitectura

- `architecture-blueprint-generator`
- `context-map`
- `create-architectural-decision-record`

Uso ideal:

- boundaries
- ownership de modulos
- decisiones de storage
- decisiones de pipeline

## 7.2 Para backlog y planeacion

- `create-implementation-plan`
- `breakdown-plan`
- `breakdown-feature-implementation`
- `breakdown-test`

Uso ideal:

- dividir epics
- ordenar entregables
- diseñar criterios de salida

## 7.3 Para documentacion y corpus

- `documentation-writer`
- `doc`
- `create-specification`
- `create-readme`

Uso ideal:

- corpus tecnico
- especificaciones
- runbooks
- docs internas

## 7.4 Para seguridad y control

- `security-best-practices`
- `security-review`
- `codex-security:security-scan`
- `codex-security:validation`

Uso ideal:

- guardrails
- permisos
- egress
- acciones R3/R4

## 7.5 Para refactor e integracion

- `refactor-plan`
- `review-and-refactor`
- `make-repo-contribution`

Uso ideal:

- integrar el nuevo pipeline
- aislar deuda tecnica
- evitar regressions estructurales

## 7.6 Para autonomia y decision engine

- `structured-autonomy-plan`
- `structured-autonomy-implement`
- `structured-autonomy-generate`

Uso ideal:

- decision engine
- action gating
- simulacion vs ejecucion
- ciclos autonomos controlados

## 7.7 Para testing y validacion

- `breakdown-test`
- `polyglot-test-agent`
- `pytest-coverage`

Uso ideal:

- matrices de prueba
- cobertura funcional
- validacion por fases

---

## 8. Skills prioritarias por orden real de uso

Si hubiera que ejecutar este proyecto con maxima eficiencia, el orden de skills mas util seria:

1. `context-map`
2. `architecture-blueprint-generator`
3. `create-implementation-plan`
4. `breakdown-feature-implementation`
5. `create-specification`
6. `documentation-writer`
7. `security-best-practices`
8. `refactor-plan`
9. `structured-autonomy-plan`
10. `breakdown-test`
11. `security-review`
12. `review-and-refactor`
13. `codex-security:validation`
14. `polyglot-test-agent`

---

## 9. Recomendacion de revision por gate

Cada fase debe cerrarse con revision formal.

## Gate A - Revision de arquitectura

Validar:

- boundaries correctos
- ausencia de acoplamiento excesivo
- feature flag disponible

## Gate B - Revision de datos

Validar:

- entidades coherentes
- seeds consistentes
- sin mezcla de transcript y memoria tecnica

## Gate C - Revision de precision

Validar:

- retrieval exacto
- ruteo correcto de especialidad
- confidence razonable

## Gate D - Revision de seguridad

Validar:

- risk policy
- snapshot requirements
- herramientas sensibles bloqueadas

## Gate E - Revision operacional

Validar:

- verificacion post-tool
- trazabilidad
- memoria util

---

## 10. Recomendacion final de ejecucion

No intentar desarrollar todas las fases en paralelo.

La secuencia correcta de implementacion para revision seria:

1. Fase 0
2. Fase 1
3. Fase 2
4. Fase 3
5. Fase 4
6. Fase 5
7. Fase 6
8. Gate tecnico
9. Fase 7
10. Fase 8
11. Fase 9
12. Fase 10
13. Fase 11
14. Gate operativo
15. Fase 12
16. Fase 13
17. Fase 14

---

## 11. Dictamen final

El desarrollo perfecto de este RAG para KernelIA requiere tres cosas al mismo tiempo:

- arquitectura correcta;
- conocimiento tecnico curado;
- disciplina de seguridad y verificacion.

Las skills no reemplazan el diseño, pero permiten ejecutar cada fase con mejor calidad.
La combinacion correcta de backlog + skills + gates de revision es lo que puede convertir a KernelIA en un agente tecnico realmente inteligente, consistente y seguro.
