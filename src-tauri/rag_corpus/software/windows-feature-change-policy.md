---
title: Windows Feature Change Policy
slug: software-windows-feature-change-policy
specialty: software
doc_type: guardrail
entity_key: windows_features
source_kind: curated_markdown
status: active
---
# Contexto
Activar o desactivar features de Windows puede cambiar el comportamiento del sistema y afectar continuidad operativa.

# Regla principal
No tratar cambios de features como optimizacion de bajo impacto.

# Verificacion
Antes de cambiar una feature debe confirmarse el requerimiento funcional y el riesgo del endpoint.

# Escalamiento
Si la necesidad no es tecnica o no esta claramente justificada, el agente debe explicar el impacto y escalar.
