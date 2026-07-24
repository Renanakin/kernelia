# FASE 12 IMPLEMENTADA

Fecha: 2026-07-12

## Objetivo

Incorporar retrieval semantico e hibrido al nucleo RAG de KernelIA sin depender de servicios externos, manteniendo operacion offline, reproducible y segura.

## Implementado

- generacion local de embeddings deterministas orientados a dominio Windows;
- persistencia de embeddings por chunk en `knowledge_chunk_embedding`;
- limpieza de embeddings anteriores durante reingesta;
- re-ranking hibrido en retrieval de conocimiento:
  - lexical;
  - semantic/cosine;
  - bonus por especialidad;
  - bonus por entidad dominante;
  - penalizacion por ambiguedad;
- prueba de recall para consultas abiertas de resolucion DNS.

## Archivos principales

- `src-tauri/src/rag/retrieval/mod.rs`
- `src-tauri/src/rag/ingest/mod.rs`
- `src-tauri/src/ai/knowledge_retriever.rs`

## Validacion esperada

- queries exactas siguen funcionando;
- queries abiertas encuentran chunks correctos aunque no repitan el termino exacto;
- la base RAG conserva embeddings persistidos por chunk.

## Resultado

KernelIA ahora dispone de retrieval hibrido real para conocimiento curado, mejorando recall semantico sin requerir una API externa de embeddings.
