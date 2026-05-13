# Fase 1: Estabilidad y Fundaciones (Ejecución)

## Objetivo
Reducir fallos visibles al usuario y dejar una base técnica verificable para iterar sin regresiones.

## Entregables de esta iteración
- Normalización central de errores de IA/API (`429`, cuota, rate limit).
- Corrección de textos visibles críticos con codificación UTF-8.
- Pruebas automáticas mínimas para la capa de normalización de errores.
- Validación técnica de build frontend y compilación backend.

## Criterios de aceptación
- El chat no muestra errores crudos de cuota; muestra mensaje amigable.
- No aparecen textos dañados en input/header principales.
- `pnpm test`, `pnpm run build` y `cargo check` pasan.

## Siguientes pasos inmediatos (Fase 1.2)
- Instrumentar trazas estructuradas de errores por módulo.
- Unificar manejo de timeouts/reintentos en todas las invocaciones `invoke`.
- Agregar pruebas de integración para flujo de streaming y tools.
