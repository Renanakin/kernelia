# Estándar de implementación IA RAG tipo Astra reutilizable para otros proyectos

Fecha: 2026-07-12

Proyecto base analizado: Hackteck Astra

Tipo de documento: referencia de arquitectura + guía de implementación

Audiencia objetivo:

- arquitectos de software
- líderes técnicos
- equipos backend
- equipos de producto que necesiten una capa de IA operativa y repetible

## 1. Objetivo del estándar

Definir una lógica de IA reutilizable para cualquier proyecto que necesite:

- responder con información del negocio y no solo con conocimiento general del modelo;
- analizar cómo responder según el plan comercial, el tipo de usuario y el contexto de la consulta;
- capturar señales de negocio durante la conversación;
- degradar de forma segura cuando el contexto es débil;
- dejar trazabilidad suficiente para auditoría, QA y tuning.

Este estándar toma como referencia la implementación actual de Astra y la abstrae como patrón reusable.

## 2. Decisión central

La implementación correcta no es:

- prompt largo + modelo + embeddings.

La implementación correcta es:

- pipeline orquestado de decisión.

En este estándar, la IA no “improvisa” libremente. La IA opera dentro de un flujo con etapas explícitas:

1. validación
2. normalización
3. análisis de consulta
4. retrieval
5. scoring y confianza
6. decisión de respuesta
7. inferencia controlada
8. tool calling
9. persistencia de memoria
10. trazabilidad

## 3. Principios obligatorios

### 3.1 Local-first

La IA debe consultar primero el conocimiento interno del negocio.

Solo después, y bajo política explícita, puede considerar fuentes externas.

### 3.2 Confidence before answer

Si la recuperación de contexto es débil, la IA no debe cerrar una recomendación. Debe pedir precisión.

### 3.3 Business-aware response

La respuesta no debe construirse solo por similitud semántica. Debe considerar:

- el dominio del negocio;
- la etapa comercial del usuario;
- la intención detectada;
- si la conversación está lista para conversión, exploración o calificación.

### 3.4 Traceability by design

Cada respuesta relevante debe poder auditarse:

- qué query se analizó;
- qué contexto se recuperó;
- por qué se respondió;
- por qué se pidió aclaración;
- qué memoria o datos quedaron persistidos.

### 3.5 Safe degradation

Si falla el retrieval, el modelo, una tool o una integración externa, el sistema debe seguir operativo con respuesta controlada.

## 4. Arquitectura lógica estándar

```text
Cliente
  -> API de chat
  -> validación y rate limit
  -> análisis de consulta
  -> retrieval híbrido
  -> evaluación de confianza
  -> decisión:
       a) aclarar
       b) responder con contexto
       c) responder sin contexto
       d) escalar a humano
  -> inferencia LLM
  -> tool calling opcional
  -> persistencia de conversación y memoria
  -> trazabilidad
```

## 5. Capas del estándar

## 5.1 Capa 1: ingreso seguro

Esta capa existe para impedir que el pipeline trabaje con entrada insegura o inútil.

Responsabilidades:

- validar esquema del request;
- limitar longitud del mensaje;
- validar historial;
- validar tokens de sesión;
- aplicar rate limit por cliente;
- sanitizar contenido antes de persistir.

Patrón Astra observado:

- `chatSchema` limita mensaje, historial, tokens y modo debug.
- existe rate limiting antes de la inferencia.
- la sesión se protege con token hash y no solo con id.

Estándar reusable:

- nunca enviar texto arbitrario al modelo sin validación previa;
- nunca persistir ids de sesión sin prueba de posesión;
- nunca permitir debug interno en producción pública.

## 5.2 Capa 2: análisis de consulta

Esta capa decide cómo tratar la consulta antes de recuperar contexto.

Debe producir:

- texto normalizado;
- tokens;
- tokens significativos;
- categoría de query;
- hint de dominio;
- coincidencias exactas de servicios o capacidades;
- necesidad de aclaración previa.

Categorías mínimas recomendadas:

- `specific`
- `short`
- `ambiguous`
- `commercial`
- `out_of_domain`

Hints de dominio recomendados:

- producto / web / SaaS
- automatización / IA
- ciberseguridad
- soporte / infraestructura
- comercial
- general

Qué resuelve esta capa:

- evita responder demasiado rápido a mensajes como “crm”, “precio”, “una web”, “ia”;
- evita que el retrieval vectorial arrastre contexto incorrecto con queries pobres;
- convierte un chatbot reactivo en un asistente con criterio.

## 5.3 Capa 3: retrieval híbrido

El estándar no debe depender de embeddings solamente.

Debe combinar:

- retrieval vectorial;
- retrieval léxico;
- ranking híbrido;
- cohesión por dominio o entidad dominante.

### 5.3.1 Retrieval vectorial

Se usa para:

- intención descriptiva;
- lenguaje natural;
- preguntas abiertas;
- variaciones semánticas.

### 5.3.2 Retrieval léxico

Se usa para:

- nombres exactos;
- slugs;
- títulos;
- siglas;
- consultas cortas;
- typos leves;
- términos comerciales concretos.

### 5.3.3 Fusión híbrida

El score final no debe ser un promedio ciego.

Debe considerar:

- peso vectorial base;
- peso léxico;
- bonus por coincidencia exacta;
- bonus por anclaje fuerte en título o slug;
- ajuste por cohesión con la entidad principal.

### 5.3.4 Regla de cohesión

Si el top match ancla fuertemente un servicio o entidad, los fragmentos del mismo servicio deben ganar prioridad relativa.

Esto evita respuestas mezcladas entre servicios similares pero distintos.

## 5.4 Capa 4: evaluación de confianza

La IA no debe usar contexto solo porque existe.

Debe calcular una señal explícita de confianza.

Salida mínima:

- `high`
- `medium`
- `low`

Además debe devolver:

- score numérico;
- razón de la decisión;
- si se debe usar contexto;
- si se debe pedir aclaración.

Señales recomendadas:

- similitud top;
- coincidencia exacta de servicio;
- especificidad de la consulta;
- penalización por query corta;
- penalización por ambigüedad;
- penalización por comercial sin objeto concreto.

Regla estándar:

- `high`: responder con contexto;
- `medium`: responder con contexto parcial y pedir un dato concreto si falta definición;
- `low`: no usar contexto y reconducir con aclaración.

## 5.5 Capa 5: orquestación de respuesta

Aquí se toma la decisión final de flujo.

Los caminos estándar son:

- `clarify`
- `infer`
- `handoff`
- `fail-safe`

### `clarify`

Usar cuando:

- la query es corta;
- la intención comercial es amplia;
- el contexto no supera el umbral;
- la pregunta es ambigua.

### `infer`

Usar cuando:

- hay contexto suficiente;
- el dominio está identificado;
- la respuesta puede darse con precisión razonable.

### `handoff`

Usar cuando:

- el usuario requiere cotización formal;
- se detecta alta intención comercial;
- se necesita un humano para cerrar alcance o precio;
- hay límites de compliance o seguridad.

### `fail-safe`

Usar cuando:

- el modelo falla;
- el retrieval falla;
- una integración externa no responde;
- el contexto queda vacío y no corresponde improvisar.

## 5.6 Capa 6: prompt de sistema orientado a negocio

El prompt no debe ser un bloque genérico.

Debe incluir:

- identidad del asistente;
- servicios o capacidades del negocio;
- tono y idioma;
- criterios de prudencia;
- instrucción de no inventar;
- instrucción de redirigir a humano cuando falte certeza;
- instrucción de capturar contacto cuando el usuario lo proporcione.

Estándar:

- el prompt base debe ser corto, estable y gobernado;
- el contexto recuperado debe inyectarse aparte;
- las decisiones de negocio no deben depender solo del prompt; deben venir del pipeline.

## 5.7 Capa 7: tool calling

Las tools no son accesorias. Son parte del valor operacional.

El estándar debe soportar al menos tres tipos de tools:

- captura de lead o contacto;
- acciones del negocio;
- integraciones seguras.

Ejemplos reutilizables:

- `registrar_contacto`
- `crear_ticket`
- `agendar_demo`
- `crear_oportunidad`
- `consultar_estado`

Reglas:

- la tool debe dispararse solo bajo esquema explícito;
- la tool no debe ejecutarse por parsing informal del texto;
- toda tool debe devolver respuesta verificable;
- el resultado de tool debe volver a la conversación si corresponde.

## 5.8 Capa 8: memoria de negocio

La mayoría de los asistentes fallan porque guardan conversación pero no memoria útil.

El estándar debe construir memoria estructurada y no solo transcript.

Campos mínimos:

- resumen de sesión;
- hechos relevantes;
- etiquetas;
- intención más reciente;
- etapa comercial;
- timestamp de última actualización.

### 5.8.1 Tipos de memoria recomendados

- `summary`
- `facts`
- `tags`
- `stage`
- `latestIntent`

### 5.8.2 Ejemplos de hechos

- CRM mencionado
- presupuesto referido
- plazo referido
- interés en demo
- stack existente
- problema principal

### 5.8.3 Ejemplos de tags

- automatización
- IA
- ventas
- SaaS
- ciberseguridad
- soporte-ti
- cloud
- urgente

### 5.8.4 Etapas comerciales sugeridas

- `DESCUBRIMIENTO`
- `EXPLORACION`
- `CALIFICADO`
- `LISTO_PARA_COTIZAR`

La memoria debe servir para:

- mejorar la continuidad conversacional;
- priorizar leads;
- derivar a ventas;
- alimentar dashboards o CRM;
- soportar handoff a humanos.

## 5.9 Capa 9: corpus de conocimiento

El estándar reusable no debe indexar solo descripciones de producto.

Debe estructurar el conocimiento por tipos de documento.

Tipos recomendados:

- overview
- propuesta de valor
- capacidades
- comparativas
- casos de uso
- preguntas de calificación
- FAQ

El patrón correcto es:

- cada servicio o capacidad del negocio se transforma en varios documentos cortos y curados;
- cada documento tiene función conversacional específica.

Esto permite que la IA responda no solo “qué es el servicio”, sino también:

- cuándo aplica;
- a qué cliente le sirve;
- cómo calificarlo;
- cómo compararlo;
- cómo manejar objeciones.

## 5.10 Capa 10: trazabilidad

Toda implementación reusable debe emitir trazas estructuradas.

Campos mínimos:

- `traceId`
- `conversationId`
- query original
- query normalizada
- categoría
- dominio detectado
- matches recuperados
- latencias por etapa
- score de confianza
- decisión tomada
- uso o no de contexto
- intento o no de retrieval externo

Esto habilita:

- debugging serio;
- tuning de thresholds;
- QA reproducible;
- evidencia para seguridad y compliance.

## 5.11 Capa 11: política de retrieval externo

La web o fuentes externas deben ser una política y no una improvisación.

Condiciones recomendadas:

- solo si falla o es insuficiente el contexto local;
- solo para queries elegibles;
- solo con proveedor configurado;
- solo si producción lo permite;
- solo si protected mode está desactivado;
- siempre con trazabilidad.

Regla estándar:

- primero corpus local;
- después opcionalmente externo;
- nunca al revés.

## 6. Modelo lógico reusable

## 6.1 Entidades mínimas

### Conversación

- id
- token hash
- timestamps
- resumen
- hechos detectados
- tags detectados
- etapa
- intención
- lead vinculado

### Mensaje

- id
- sessionId
- role
- content
- createdAt

### Lead o entidad de negocio

- id
- nombre
- email
- teléfono
- tags comerciales
- hechos relevantes
- resumen IA
- etapa
- owner opcional

### Conocimiento

- id o slug
- entidad principal
- tipo de fragmento
- título
- descripción
- cuerpo
- embedding
- versión
- hash de contenido

## 6.2 Separación que debe respetarse

- transcript conversacional
- memoria operacional
- lead o entidad comercial
- corpus de conocimiento
- observabilidad

No mezclar estas capas simplifica mantenimiento y portabilidad.

## 7. Estándar de decisión de respuesta

La respuesta debe obedecer esta matriz:

| Estado | Contexto | Acción |
|---|---|---|
| Query corta + confianza baja | No usable | pedir precisión |
| Query ambigua + confianza baja | No usable | pedir precisión |
| Query específica + confianza alta | usable | responder con contexto |
| Query específica + confianza media | parcial | responder con cautela |
| Query fuera de dominio | irrelevante | reconducir o limitar alcance |
| Query comercial sin objeto claro | insuficiente | calificar antes de recomendar |
| Usuario entrega contacto | no depende del retrieval | capturar mediante tool |

## 8. Estándar de análisis según plan de negocio

Esta es la parte más importante si el objetivo es replicar la lógica en otros proyectos.

La IA no debe responder solo en función de “qué sabe”, sino en función de “cómo vende, califica y orienta el negocio”.

Cada proyecto debe definir un playbook comercial estructurado con:

- líneas de negocio;
- perfil de cliente ideal;
- dolores frecuentes;
- criterios de calificación;
- objeciones frecuentes;
- señales de intención alta;
- criterios de derivación a humano;
- límites de lo que la IA puede prometer.

## 8.1 Artefactos de negocio obligatorios

Para replicar Astra en otro proyecto se debe producir, antes de implementar, lo siguiente:

1. catálogo de servicios o capacidades
2. mapa de casos de uso
3. matriz de preguntas de descubrimiento
4. reglas de clasificación comercial
5. matriz de intención
6. matriz de etapas del lead
7. FAQ comercial
8. comparativas o diferenciales

Sin estos artefactos, el RAG responde, pero no vende ni califica bien.

## 8.2 Matriz mínima por línea de negocio

Cada línea debe tener:

- qué resuelve
- para quién aplica
- cuándo no aplica
- cómo detectarla en lenguaje natural
- qué preguntar para calificarla
- qué señales indican urgencia
- qué datos mínimos pide ventas para avanzar

## 9. Proceso estándar para portar este modelo a otro proyecto

## Fase 1: discovery de negocio

Objetivo:

- convertir el conocimiento comercial en estructura usable por IA.

Entregables:

- mapa de servicios;
- taxonomía de dominios;
- intents;
- stages;
- criterios de derivación.

## Fase 2: diseño de corpus

Objetivo:

- fragmentar conocimiento de negocio en documentos curados.

Entregables:

- overview por capacidad;
- FAQs;
- qualification prompts;
- comparativas;
- casos de uso.

## Fase 3: diseño del pipeline

Objetivo:

- implementar el flujo de análisis, retrieval, scoring y decisión.

Entregables:

- analyzer;
- retriever híbrido;
- confidence assessor;
- decision router.

## Fase 4: integración LLM

Objetivo:

- agregar inferencia, prompt gobernado y tool calling.

Entregables:

- system prompt estable;
- tools con contratos;
- fallbacks de modelo;
- respuestas controladas.

## Fase 5: memoria y persistencia

Objetivo:

- convertir conversación en señal operacional.

Entregables:

- sesión;
- transcript;
- snapshot comercial;
- lead linkage.

## Fase 6: observabilidad

Objetivo:

- hacer auditable el pipeline.

Entregables:

- trace logs;
- métricas por etapa;
- debug mode interno;
- panel de revisión QA.

## Fase 7: hardening

Objetivo:

- asegurar que la IA sea explotable operativamente sin degradar seguridad.

Entregables:

- rate limiting;
- sanitización;
- políticas de retención;
- bloqueo de debug en producción;
- política de egress.

## 10. Contratos de módulos recomendados

Para volver esto estándar, cada implementación debe exponer módulos similares.

### `analyzeQuery(input)`

Retorna:

- normalized
- tokens
- significantTokens
- category
- domainHint
- exact matches
- clarificationPrompt

### `retrieveKnowledge(query, limit)`

Retorna:

- context
- matches
- telemetry

### `assessConfidence(queryAnalysis, matches)`

Retorna:

- level
- score
- reason
- shouldUseContext
- shouldAskClarifyingQuestion

### `buildPromptContext(decision, ragResult)`

Retorna:

- contexto final inyectable al modelo

### `buildCommercialMemorySnapshot(input)`

Retorna:

- summary
- facts
- tags
- stage
- latestIntent

### `buildTrace(...)`

Retorna:

- objeto estructurado de observabilidad

## 11. Métricas que deben existir en cualquier implementación

Métricas mínimas:

- tasa de respuestas con contexto
- tasa de aclaraciones
- tasa de queries fuera de dominio
- precisión percibida en QA
- latencia de análisis
- latencia de retrieval vectorial
- latencia de retrieval léxico
- latencia total
- porcentaje de handoff a humano
- porcentaje de capturas de lead

Métricas de negocio:

- consultas que pasan a `CALIFICADO`
- consultas que pasan a `LISTO_PARA_COTIZAR`
- leads con contacto capturado
- intención dominante por línea de negocio
- top objeciones o necesidades detectadas

## 12. Anti-patrones que este estándar evita

- chatbot con prompt gigante y sin retrieval
- RAG solo vectorial
- usar contexto aunque sea débil
- guardar solo transcript sin memoria útil
- mezclar lógica comercial dentro del prompt
- tool calling sin esquema ni validación
- usar web antes del corpus interno
- no saber por qué respondió algo
- responder “precio” sin antes calificar

## 13. Recomendación de implementación base para cualquier otro proyecto

Si se quiere replicar este modelo en otro negocio, la secuencia correcta es:

1. definir dominios del negocio
2. estructurar corpus por tipo de documento
3. implementar analyzer
4. implementar retriever híbrido
5. implementar confidence gating
6. implementar router de respuesta
7. implementar tools del negocio
8. implementar memoria comercial
9. implementar trazabilidad
10. recién después considerar fuentes externas

## 14. Qué debe parametrizarse al portar el estándar

Partes reutilizables sin cambios mayores:

- pipeline
- categorías de query
- modelo de confianza
- estructura de trace
- memoria comercial básica
- patrón local-first

Partes que sí deben cambiar por proyecto:

- taxonomía de dominios
- catálogo de servicios
- preguntas de calificación
- reglas de intención
- FAQ
- comparativas
- stages comerciales si el negocio requiere otros
- tools disponibles

## 15. Dictamen final

La lógica de Astra/RAG sí puede convertirse en estándar reusable para otros proyectos, pero debe replicarse como arquitectura de decisión y no solo como integración con un LLM.

El estándar correcto queda definido así:

- RAG híbrido
- análisis de query
- gating por confianza
- respuesta gobernada por plan de negocio
- tool calling tipado
- memoria comercial persistente
- trazabilidad obligatoria
- política de egress controlada

Ese es el patrón que permite que la IA:

- responda mejor;
- califique mejor;
- convierta mejor;
- sea auditable;
- y pueda portarse a otros contextos sin rehacer toda la lógica desde cero.

