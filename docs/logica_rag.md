# Documento de implementación: Advanced RAG para análisis e implementación por agentes

## Propósito

Este documento describe una arquitectura de **Advanced RAG** inspirada en el enfoque mostrado por Hugging Face para construir un sistema de preguntas y respuestas sobre una base documental usando LangChain.[page:1] El objetivo es que un agente técnico pueda usar este documento como especificación operativa para analizar, planificar e implementar un sistema RAG robusto, medible y extensible.[page:1]

## Objetivo funcional

El sistema debe responder preguntas de usuarios sobre una base de conocimiento específica mediante dos etapas principales: **retrieval** de fragmentos relevantes y **generation** de una respuesta usando un LLM lector.[page:1] La receta de Hugging Face enfatiza que el rendimiento depende de varios puntos de ajuste, incluyendo chunking, embeddings, base vectorial, reranking, prompt y evaluación iterativa.[page:1]

## Arquitectura objetivo

La arquitectura propuesta se compone de los siguientes módulos:[page:1]

1. **Carga de knowledge base**: ingestión de documentos fuente y conservación de metadatos como `source`.[page:1]
2. **Preprocesamiento y chunking**: división de documentos en fragmentos semánticamente útiles usando `RecursiveCharacterTextSplitter` con separadores orientados a Markdown.[page:1]
3. **Embeddings**: codificación de cada chunk con un modelo de embeddings compartido para documentos y consultas; el ejemplo usa `thenlper/gte-small`.[page:1]
4. **Índice vectorial**: almacenamiento de embeddings en FAISS con estrategia de distancia cosine y embeddings normalizados.[page:1]
5. **Retriever inicial**: recuperación de un conjunto amplio de documentos candidatos usando similarity search.[page:1]
6. **Reranker opcional**: reordenamiento de candidatos con un modelo más potente, en el ejemplo `colbert-ir/colbertv2.0` mediante RAGatouille.[page:1]
7. **Construcción de contexto**: ensamblado de los documentos finales en un prompt estructurado para el LLM lector.[page:1]
8. **Reader LLM**: generación de la respuesta final con un modelo causal; el ejemplo usa `HuggingFaceH4/zephyr-7b-beta` cuantizado a 4 bits.[page:1]
9. **Evaluación continua**: medición iterativa del sistema sobre un dataset pequeño de preguntas/respuestas antes de optimizar componentes.[page:1]

## Flujo de procesamiento

### 1. Ingesta

La base documental se carga como un dataset y luego se convierte en documentos LangChain, preservando `page_content` y `metadata.source`.[page:1] Este detalle es crítico porque el sistema de respuesta y trazabilidad depende de poder citar el documento original o al menos mantener una referencia de procedencia.[page:1]

### 2. Chunking

La guía recomienda chunking recursivo para preservar estructura global y adaptarse mejor a documentos Markdown.[page:1] También propone una lista de separadores jerárquicos que prioriza encabezados, bloques de código, separadores horizontales, saltos de línea dobles, saltos simples, espacios y, como último recurso, partición por longitud.[page:1]

El documento original advierte que definir chunk size en caracteres puede romper la compatibilidad con el límite real del modelo de embeddings, por lo que la versión mejorada cambia la medición a **tokens** usando el tokenizer del modelo de embeddings.[page:1] El ejemplo ajusta el chunk size a 512 tokens y usa overlap equivalente a una décima parte del chunk size para reducir cortes semánticos entre fragmentos adyacentes.[page:1]

### 3. Control de longitud

La receta consulta explícitamente `max_seq_length` del modelo de embeddings y muestra que `thenlper/gte-small` tiene un máximo de 512 tokens.[page:1] Por ello, los chunks deben mantenerse por debajo de ese límite, porque cualquier exceso será truncado durante embedding y perderá relevancia recuperable.[page:1]

### 4. Deduplicación

Después del split, el ejemplo elimina chunks duplicados comparando `page_content`.[page:1] Esta deduplicación evita inflar el índice, reduce ruido de recuperación y simplifica la etapa de reranking.[page:1]

### 5. Embeddings e índice vectorial

La guía usa `HuggingFaceEmbeddings` con normalización de embeddings activada y FAISS como vector store.[page:1] También indica que, con cosine similarity, es necesario normalizar embeddings, y que FAISS es una elección pragmática por rendimiento y adopción amplia.[page:1]

### 6. Retrieval

El retriever actúa como buscador interno: recibe una query, la convierte con el mismo embedding model y recupera los vecinos más cercanos del índice vectorial.[page:1] El parámetro `top_k` controla cuántos fragmentos se recuperan, pero la guía advierte que incrementar demasiado el contexto puede degradar al reader por el fenómeno **lost in the middle**, especialmente cuando el contexto agregado excede aproximadamente 16k tokens en muchos modelos actuales.[page:1]

### 7. Reranking

La guía recomienda recuperar más documentos de los que se usarán finalmente y luego rerankearlos con un modelo más fuerte.[page:1] El ejemplo propone ColBERTv2 vía RAGatouille porque modela interacciones más finas entre tokens de query y documento que un bi-encoder clásico.[page:1]

### 8. Reader y prompt

La etapa reader toma el contexto consolidado y la pregunta del usuario dentro de un prompt con formato de chat.[page:1] El prompt de ejemplo impone tres restricciones esenciales: responder solo a la pregunta, mantener la respuesta concisa y relevante, y no responder cuando la respuesta no pueda deducirse del contexto.[page:1]

### 9. Respuesta final

La función de ensamblaje presentada por Hugging Face sigue esta secuencia: recuperar documentos, rerankear opcionalmente, construir contexto, formatear prompt y generar respuesta con el LLM.[page:1] La implementación propuesta para producción debe preservar esta secuencia, pero aislar cada paso como componente intercambiable para facilitar tuning, testing y observabilidad.[page:1]

## Especificación técnica por componentes

### Módulo A: Document loader

**Responsabilidad**: cargar documentos fuente desde repositorios, archivos, CMS o datasets y estandarizarlos como objetos documento con contenido y metadata.[page:1]

**Contrato sugerido**:

```ts
interface KBChunkSource {
  id: string
  content: string
  source: string
  title?: string
  section?: string
  updatedAt?: string
  tags?: string[]
}
```

**Reglas**:
- Mantener `source` obligatorio para trazabilidad.[page:1]
- Incluir `title`, `section` y offsets cuando sea posible para mejorar debugging y UX.
- Versionar el origen cuando la documentación cambie con frecuencia.

### Módulo B: Splitter

**Responsabilidad**: convertir documentos completos en chunks semánticamente útiles y compatibles con el embedding model.[page:1]

**Requisitos operativos**:
- Medir chunk size en tokens, no en caracteres.[page:1]
- Configurar overlap aproximado al 10% del chunk size.[page:1]
- Usar separadores específicos para Markdown cuando la fuente sea documentación técnica.[page:1]
- Guardar `start_index` u offset equivalente para poder reconstruir ubicación original.[page:1]
- Eliminar duplicados exactos después del split.[page:1]

**Pseudocódigo**:

```python
def split_documents(docs, tokenizer_name, chunk_size):
    splitter = RecursiveCharacterTextSplitter.from_huggingface_tokenizer(
        AutoTokenizer.from_pretrained(tokenizer_name),
        chunk_size=chunk_size,
        chunk_overlap=int(chunk_size / 10),
        add_start_index=True,
        strip_whitespace=True,
        separators=MARKDOWN_SEPARATORS,
    )

    chunks = []
    for doc in docs:
        chunks.extend(splitter.split_documents([doc]))

    return deduplicate_by_content(chunks)
```

### Módulo C: Embedding service

**Responsabilidad**: producir embeddings para documentos y queries con el mismo modelo y parámetros.[page:1]

**Requisitos**:
- Un único modelo compartido entre indexación y búsqueda.[page:1]
- Si se usa cosine similarity, activar normalización de embeddings.[page:1]
- Exponer `embed_documents()` y `embed_query()`.
- Registrar versión del modelo en metadatos del índice.

**Modelo de referencia del ejemplo**: `thenlper/gte-small`.[page:1]

### Módulo D: Vector index

**Responsabilidad**: almacenar embeddings y ejecutar nearest neighbor search.[page:1]

**Tecnología recomendada en esta receta**: FAISS con distancia cosine.[page:1]

**Operaciones mínimas**:
- `build_index(chunks)`
- `save_index(path)`
- `load_index(path)`
- `similarity_search(query, k)`
- `rebuild_if_embedding_version_changed()`

### Módulo E: Retriever

**Responsabilidad**: obtener un conjunto inicial amplio de candidatos relevantes.[page:1]

**Parámetros sugeridos**:
- `num_retrieved_docs`: entre 20 y 50 como rango de tuning inicial si habrá reranker; el ejemplo usa 30.[page:1]
- `num_docs_final`: entre 3 y 8 según presupuesto de contexto; el ejemplo usa 5.[page:1]

**Criterio de diseño**: recuperar más candidatos de los que se consumirán finalmente cuando exista reranker, para mejorar recall en la primera etapa.[page:1]

### Módulo F: Reranker

**Responsabilidad**: refinar el orden de los documentos recuperados para mejorar precisión antes del contexto final.[page:1]

**Tecnología de referencia**: `colbert-ir/colbertv2.0` cargado con RAGatouille.[page:1]

**Contrato sugerido**:

```ts
interface RerankedDoc {
  content: string
  score: number
  source?: string
}
```

**Cuándo activarlo**:
- Documentación extensa.
- Preguntas ambiguas o de alta precisión.
- Casos donde el embedding retrieval simple devuelve contexto correcto pero mal ordenado.[page:1]

### Módulo G: Prompt builder

**Responsabilidad**: transformar documentos finales y pregunta del usuario en un prompt consistente con el chat template del modelo lector.[page:1]

**Instrucciones mínimas observadas en la receta**:[page:1]
- Responder usando la información del contexto.
- Responder solo a la pregunta.
- Mantener concisión y relevancia.
- Referenciar número de documento cuando sea pertinente.
- No inventar respuesta si no puede deducirse del contexto.

**Recomendación de implementación**:
- Separar instrucciones del sistema y contexto del usuario.
- Numerar documentos del contexto.
- Adjuntar metadatos útiles para cita o linking en capa de aplicación.
- Limitar tokens del contexto antes de invocar al LLM.

### Módulo H: Reader LLM

**Responsabilidad**: redactar la respuesta final usando pregunta y contexto.[page:1]

**Modelo de referencia**: `HuggingFaceH4/zephyr-7b-beta` cuantizado con BitsAndBytes a 4 bits (`nf4`, `bfloat16`).[page:1] La guía señala que el reader debe tener una ventana suficiente para acomodar prompt y contexto, estimando al menos 4k tokens cuando se usan 5 documentos de hasta 512 tokens cada uno.[page:1]

**Parámetros del ejemplo**:
- `task="text-generation"`.[page:1]
- `temperature=0.2`.[page:1]
- `repetition_penalty=1.1`.[page:1]
- `max_new_tokens=500`.[page:1]

## Pipeline de referencia

```python
def answer_with_rag(question, llm, knowledge_index, reranker=None,
                    num_retrieved_docs=30, num_docs_final=5):
    relevant_docs = knowledge_index.similarity_search(
        query=question,
        k=num_retrieved_docs
    )

    relevant_docs = [doc.page_content for doc in relevant_docs]

    if reranker:
        relevant_docs = reranker.rerank(question, relevant_docs, k=num_docs_final)
        relevant_docs = [doc["content"] for doc in relevant_docs]

    relevant_docs = relevant_docs[:num_docs_final]

    context = "\nExtracted documents:\n" + "".join([
        f"Document {i}:::\n{doc}" for i, doc in enumerate(relevant_docs)
    ])

    final_prompt = RAG_PROMPT_TEMPLATE.format(
        question=question,
        context=context
    )

    answer = llm(final_prompt)["generated_text"]
    return answer, relevant_docs
```

Este flujo refleja de forma directa el ensamblaje mostrado en la guía de Hugging Face.[page:1] Para una implementación real, conviene desacoplarlo en servicios independientes con telemetría, caché, timeouts y manejo explícito de errores.[page:1]

## Decisiones de diseño recomendadas para producción

### 1. Separar indexación y serving

La receta demuestra la construcción del índice y la inferencia dentro de un notebook único.[page:1] En producción, conviene dividirlo en dos pipelines: **offline indexing** para ingesta/reindexado y **online query serving** para responder consultas con baja latencia.

### 2. Mantener metadata rica

El ejemplo conserva `source` y `start_index`.[page:1] En una implementación empresarial, se recomienda ampliar metadata con `document_id`, `section_title`, `breadcrumb`, `version`, `language`, `tenant_id` y `access_scope` para soportar seguridad, filtros y UX de citas.

### 3. Diseñar para reemplazo de componentes

La guía subraya que muchos pasos son ajustables: chunking, modelo de embeddings, índice, reranker, prompt y reader.[page:1] Por eso la arquitectura debe ser componible y no acoplar FAISS, GTE, ColBERT o Zephyr a interfaces rígidas.

### 4. Presupuestar contexto

Hugging Face advierte sobre el riesgo de saturar al reader con demasiado contexto y menciona el fenómeno lost-in-the-middle.[page:1] La implementación debe imponer un **budget de tokens** por respuesta y recortar contexto por score, diversidad o compresión antes del prompt final.

### 5. Evaluar antes de optimizar

La guía recomienda comenzar construyendo un pequeño dataset de evaluación y luego iterar con cambios pequeños para medir impacto.[page:1] Esto implica que toda optimización debe medirse con métricas repetibles y no solo por inspección manual.

## Plan de implementación por fases

### Fase 1: MVP funcional

Objetivo: disponer de un sistema end-to-end mínimo pero correcto.[page:1]

**Entregables**:
- Loader documental.
- Chunking por tokens con separadores Markdown.[page:1]
- Embeddings con un modelo consistente para docs/query.[page:1]
- Índice FAISS cosine.[page:1]
- Retriever simple top-k.[page:1]
- Prompt builder básico.
- Reader LLM con respuesta basada solo en contexto.[page:1]
- Respuesta con documentos fuente asociados.

### Fase 2: Calidad de recuperación

Objetivo: mejorar recall y precisión en retrieval.[page:1]

**Entregables**:
- Detección y eliminación de chunks duplicados.[page:1]
- Ajuste fino de chunk size y overlap.[page:1]
- Benchmarks entre varios embedding models.
- Reranking con ColBERTv2/RAGatouille.[page:1]
- Filtros por metadata.

### Fase 3: Calidad de respuesta

Objetivo: mejorar grounding, fidelidad y utilidad de la respuesta.[page:1]

**Entregables**:
- Prompt con política de abstención (“no responder si no está en contexto”).[page:1]
- Citas o referencias de documento visibles para el usuario.[page:1]
- Formateo estructurado de salidas.
- Controles anti-hallucination y validación posterior.

### Fase 4: Evaluación y observabilidad

Objetivo: poder medir y operar el sistema.[page:1]

**Entregables**:
- Dataset de evaluación con preguntas reales.[page:1]
- Métricas de retrieval y answer quality.
- Logging de query, documentos recuperados, scores y latencia.
- Trazas por etapa del pipeline.
- Dashboard de drift y cobertura.

### Fase 5: Optimización avanzada

Objetivo: escalar calidad y rendimiento.[page:1]

**Entregables**:
- Query expansion.[page:1]
- Context compression.[page:1]
- Soporte conversacional.[page:1]
- Caché semántica o de respuestas.
- Reindex incremental y multi-tenant.

## Contratos de datos sugeridos

### QueryRequest

```json
{
  "question": "string",
  "user_id": "string",
  "session_id": "string",
  "filters": {
    "source": ["string"],
    "language": ["string"],
    "version": ["string"]
  },
  "top_k": 30,
  "final_k": 5
}
```

### RetrievedChunk

```json
{
  "chunk_id": "string",
  "content": "string",
  "score": 0.0,
  "source": "string",
  "start_index": 0,
  "metadata": {}
}
```

### RAGResponse

```json
{
  "answer": "string",
  "citations": [
    {
      "chunk_id": "string",
      "source": "string",
      "start_index": 0
    }
  ],
  "debug": {
    "retrieved_count": 30,
    "final_count": 5,
    "reranker_used": true,
    "reader_model": "string",
    "embedding_model": "string"
  }
}
```

## Métricas recomendadas

La guía insiste en que primero hay que medir y luego mejorar.[page:1] Con base en esa recomendación, un sistema productivo debería incorporar al menos las siguientes métricas:

### Retrieval
- Recall@k sobre dataset etiquetado.
- MRR o nDCG si existe ranking esperado.
- Cobertura por fuente documental.
- Tasa de chunks truncados por token overflow.

### Reader
- Groundedness: proporción de respuestas sustentadas en contexto.
- Faithfulness: tasa de afirmaciones alineadas con documentos recuperados.
- Answer relevancy frente a la pregunta.
- Abstention accuracy cuando no existe evidencia suficiente.

### Operación
- Latencia por etapa: embed query, search, rerank, generation.
- Tokens de entrada y salida.
- Coste por consulta.
- Ratio de caché hit/miss.

## Riesgos y mitigaciones

| Riesgo | Descripción | Mitigación |
|---|---|---|
| Chunks demasiado grandes | El embedding model puede truncar contenido y perder señal semántica.[page:1] | Medir en tokens y respetar `max_seq_length`.[page:1] |
| Chunks demasiado pequeños | Se fragmentan ideas y disminuye el valor semántico del contexto.[page:1] | Ajustar tamaño y overlap con evaluación iterativa.[page:1] |
| Exceso de contexto | El reader puede degradar por lost-in-the-middle.[page:1] | Budget de tokens, reranking y compresión previa.[page:1] |
| Retrieval impreciso | Similarity search puede recuperar material correcto pero mal priorizado.[page:1] | Recuperar más candidatos y rerankear con ColBERTv2.[page:1] |
| Hallucination | El LLM responde más allá del contexto disponible.[page:1] | Prompt de abstención y validación basada en evidencia.[page:1] |
| Optimización a ciegas | Cambios sin medición pueden empeorar el sistema.[page:1] | Dataset de evaluación y experimentación incremental.[page:1] |

## Recomendaciones de implementación para agentes

Un agente encargado de implementar este sistema debería seguir esta secuencia operativa:[page:1]

1. Inventariar fuentes y formato documental.
2. Seleccionar estrategia de chunking según estructura real del contenido, priorizando token-based chunking.[page:1]
3. Verificar límite `max_seq_length` del embedding model antes de indexar.[page:1]
4. Generar índice vectorial y validar retrieval con preguntas manuales.[page:1]
5. Integrar reranker si el recall inicial es aceptable pero la precisión final no lo es.[page:1]
6. Definir prompt de reader con política explícita de no invención.[page:1]
7. Construir dataset de evaluación antes de hacer tuning extensivo.[page:1]
8. Iterar sobre una variable por vez: chunk size, embedding model, top-k, reranker, prompt o reader.[page:1]

## Checklist de implementación

### Infraestructura
- [ ] Servicio de indexación separado del servicio de consulta.
- [ ] Persistencia del índice FAISS o equivalente.
- [ ] Gestión de versiones de embeddings e índice.
- [ ] Observabilidad por etapa.

### Datos
- [ ] Fuente documental normalizada.
- [ ] Metadata mínima: `source`, `document_id`, `start_index`.
- [ ] Deduplicación de chunks.[page:1]
- [ ] Estrategia de reindexado incremental.

### Retrieval
- [ ] Chunking por tokens.[page:1]
- [ ] Overlap parametrizable.[page:1]
- [ ] Embeddings normalizados si se usa cosine.[page:1]
- [ ] Búsqueda similarity search validada.[page:1]
- [ ] Reranker opcional activable por configuración.[page:1]

### Generation
- [ ] Prompt chat-template compatible con el modelo.[page:1]
- [ ] Política de abstención.[page:1]
- [ ] Context budgeting.
- [ ] Salida con citas y fuentes.

### Calidad
- [ ] Dataset de evaluación inicial.[page:1]
- [ ] Métricas de retrieval y reader.
- [ ] Pruebas con preguntas sin respuesta para validar abstención.[page:1]
- [ ] Pruebas de regresión por release.

## Extensiones recomendadas

La propia guía sugiere varias líneas de mejora posteriores: semantic chunking, cambio de embedding model, cambio del índice, query expansion, tuning del prompt, activar o desactivar reranking, usar un reader más potente, comprimir el contexto y volver el sistema más conversacional o con citas más visibles.[page:1] Estas extensiones deben incorporarse solo después de que exista una línea base evaluada y trazable.[page:1]

## Conclusión operativa

La receta de Hugging Face no plantea RAG como un bloque monolítico, sino como una cadena de componentes ajustables donde la calidad final depende del alineamiento entre chunking, embeddings, retrieval, reranking, prompt y reader.[page:1] Para una implementación profesional orientada a agentes, la prioridad no debe ser solo “hacer que responda”, sino construir un sistema modular, evaluable y con control explícito sobre contexto, evidencia y trazabilidad.[page:1]