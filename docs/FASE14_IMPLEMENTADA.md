# FASE 14 IMPLEMENTADA

Fecha: 2026-07-12

## Objetivo

Introducir el nuevo nucleo RAG en la UI de KernelIA con activacion gradual, comparacion tecnica y panel interno de QA.

## Implementado

- `rag_engine_enabled` como bandera persistente en `AppSettings`;
- modo comparativo para revisar:
  - intencion legacy;
  - plan legado;
  - especialidad RAG;
  - decision RAG;
  - confianza comparada;
- panel interno de debug por mensaje con:
  - especialidad;
  - confianza;
  - decision;
  - riesgo;
  - trace id;
  - conteos de retrieval;
  - reason codes;
  - conflictos de live state;
- badges visibles de confianza/especialidad cuando el RAG esta activo;
- soporte de lectura/escritura de estas opciones desde la ventana de configuracion.

## Archivos principales

- `src-tauri/src/config/settings.rs`
- `src-tauri/src/ai/router.rs`
- `src/lib/stores/settings.js`
- `src/lib/stores/chat.js`
- `src/lib/components/SettingsModal.svelte`
- `src/lib/components/MessageBubble.svelte`
- `src/lib/components/ChatWindow.svelte`
- `src/lib/components/InputBar.svelte`

## Resultado

La UI puede activar o desactivar el RAG por configuracion o entorno, comparar el flujo nuevo contra el legado y revisar la trazabilidad sin romper la experiencia principal de chat.
