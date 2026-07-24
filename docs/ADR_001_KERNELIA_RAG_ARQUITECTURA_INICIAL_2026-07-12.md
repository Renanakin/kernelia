# ADR 001 - Arquitectura Inicial del Subsistema RAG de KernelIA

Fecha: 2026-07-12

Estado: aceptado

## Contexto

KernelIA dispone hoy de:

- chat con function calling;
- tools de sistema;
- RBAC;
- auditoria;
- snapshots;
- intent engine inicial.

Pero no dispone de:

- un subsistema RAG tecnico persistente;
- un catalogo formal de conocimiento y comandos;
- un motor de decision desacoplado del LLM.

## Decision

Se crea un subsistema nuevo `src-tauri/src/rag/` y se agregan modulos nuevos en `src-tauri/src/ai/` para modelar la evolucion del nucleo hacia una arquitectura de decision tecnica.

La arquitectura inicial separa:

- modelos del dominio RAG;
- storage y migraciones;
- retrieval;
- decision;
- memoria;
- trazabilidad;
- ingesta y policies.

El flujo actual de `router.rs` y `function_calling.rs` no se reemplaza aun.
En esta fase solo se crea el boundary y los contratos base.

## Decision complementaria

El almacenamiento inicial sera local-first usando SQLite como objetivo de Fase 1.

Motivos:

- portabilidad;
- bajo costo operativo;
- alineacion con el enfoque desktop/local de KernelIA;
- facilidad para usar FTS5 en retrieval lexical;
- permite evolucionar luego a PostgreSQL/pgvector sin romper el modelo conceptual.

## Consecuencias

Positivas:

- se desacopla la futura inteligencia del flujo actual de chat;
- se define una estructura sostenible para crecer;
- se habilita trabajo por fases.

Negativas:

- se introduce complejidad estructural temprana;
- habra un periodo transitorio con logica legacy y logica nueva conviviendo.

## Limites de esta ADR

Esta ADR no decide aun:

- el proveedor de embeddings;
- si habra vector store local o remoto;
- el detalle de politicas de confidence scoring final;
- el rollout de UI.

Esas decisiones quedan para ADRs posteriores.
