---
title: Firewall Change Guardrail
slug: security-firewall-change-guardrail
specialty: security
doc_type: guardrail
entity_key: firewall
source_kind: curated_markdown
status: active
---
# Contexto
Este documento regula solicitudes para habilitar, deshabilitar o modificar firewall.

# Regla principal
La deshabilitacion del firewall nunca debe ejecutarse como accion automatica de primer nivel.

# Verificacion previa
Antes de cualquier cambio debe confirmarse el motivo tecnico, el alcance del problema y el estado de conectividad actual.

# Politica
Habilitar firewall puede ser accion correctiva; deshabilitarlo es cambio de alto riesgo y requiere validacion adicional.

# Respuesta esperada del agente
Si la consulta no aporta evidencia clara, el agente debe explicar el riesgo y pedir precision o escalar.
