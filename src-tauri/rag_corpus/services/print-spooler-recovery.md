---
title: Print Spooler Recovery
slug: services-print-spooler-recovery
specialty: services
doc_type: playbook
entity_key: spooler
source_kind: curated_markdown
status: active
---
# Contexto
Este playbook cubre incidentes donde las impresoras no imprimen, la cola queda trabada o el usuario menciona `spooler`.

# Sintomas tipicos
Trabajos congelados, impresora visible pero sin salida, errores al agregar impresoras o reinicios repetidos del servicio.

# Verificacion
Consultar el estado del servicio objetivo con `get_service_status` o `list_services` y confirmar si esta detenido, corriendo o reiniciando.

# Accion recomendada
Si el servicio esta degradado y el riesgo es aceptable, intentar `restart_service` y luego verificar estado nuevamente. No deshabilitar el servicio como primera accion.

# Guardrails
Detener o deshabilitar servicios debe tratarse como cambio operativo, nunca como diagnostico base.

# Escalamiento
Escalar si el servicio vuelve a caer despues del reinicio o si existe dependencia rota con otros servicios del sistema.
