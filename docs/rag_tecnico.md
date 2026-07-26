MANUAL TÉCNICO – LÓGICA RAG DEL ARTÍCULO (GIUSEPPE TRISCIUOGLIO)

1. OBJETIVO GENERAL

El artículo describe cómo construir un asistente de help desk que:
- Primero intenta resolver las consultas usando una base de conocimiento interna mediante RAG (Retrieval-Augmented Generation).
- Si no encuentra una solución adecuada, o el cliente no queda satisfecho, crea un ticket de soporte para un operador humano.

La idea es combinar:
- Un pipeline RAG con LangChain4j para recuperar información relevante.
- Un sistema de ticketing integrado (Redmine, Jira, GitHub Issues, etc.) para escalar casos no resueltos.

2. ARQUITECTURA RAG (KNOWLEDGE BASE)

Componentes principales de la base de conocimiento:

- EmbeddingModel
  - Se usa AllMiniLmL6V2EmbeddingModel.
  - Convierte textos en vectores numéricos (embeddings) que permiten búsqueda semántica eficiente.

- EmbeddingStore
  - Se usa InMemoryEmbeddingStore.
  - Almacena los embeddings generados.
  - Elección pensada para simplicidad y prototipado rápido.
  - A futuro se puede reemplazar por Qdrant, Weaviate u otro vector store persistente y escalable.

- ContentRetriever
  - Se usa EmbeddingStoreContentRetriever.
  - Recupera contenido relevante del EmbeddingStore usando embeddings y similitud.

- DocumentSplitter
  - Se usa DocumentBySentenceSplitter(250, 50, tokenizer).
  - Parte los documentos en chunks de tamaño ~250 tokens con solapamiento de ~50 tokens.
  - Mantiene contexto entre fragmentos evitando que la información relevante se corte mal.

- RetrievalAugmentor
  - Se usa DefaultRetrievalAugmentor.
  - Enriquece las instrucciones para el modelo LLM con el contenido recuperado (contexto).

- ContentInjector
  - Se usa DefaultContentInjector.
  - Inyecta en el prompt el contenido recuperado junto con metadatos específicos.
  - Se incluyen metadatos como "source", "relevance" e "idioma" para trazabilidad.

Beans ejemplificados en el artículo (resumen en pseudocódigo Java):

- Tokenizer:
  - HuggingFaceTokenizer.

- EmbeddingModel:
  - AllMiniLmL6V2EmbeddingModel.

- EmbeddingStore:
  - InMemoryEmbeddingStore<>.

- EmbeddingStoreIngestor:
  - Usa DocumentSplitter (DocumentBySentenceSplitter) + EmbeddingModel + EmbeddingStore.
  - Se encarga de:
    - Leer documentos.
    - Dividirlos en segmentos.
    - Generar embeddings.
    - Persistirlos en el EmbeddingStore.

- ContentRetriever:
  - Usa EmbeddingStoreContentRetriever con:
    - maxResults = 5
    - minScore = 0.6
  - Estos valores se definieron después de experimentación para equilibrar precisión y cobertura.

- ContentInjector:
  - DefaultContentInjector.builder()
    - metadataKeysToInclude(["source", "relevance", "idioma"]).

- RetrievalAugmentor:
  - DefaultRetrievalAugmentor.builder()
    - contentRetriever(contentRetriever)
    - contentInjector(contentInjector).

3. PROCESO DE INGESTA DE DOCUMENTOS

Objetivo: poblar la base de conocimiento con documentos internos.

Pasos:

1) Obtener documentos internos:
   - Manuales, FAQs, artículos, documentación técnica, etc.

2) Dividir documentos:
   - Usar DocumentSplitter (DocumentBySentenceSplitter) con tamaño y overlap adecuados.
   - Evitar chunks demasiado grandes (perderían especificidad) o demasiado pequeños (perderían contexto).

3) Generar embeddings:
   - Usar EmbeddingModel (AllMiniLmL6V2EmbeddingModel) para convertir cada chunk en un vector.

4) Guardar en EmbeddingStore:
   - Usar EmbeddingStoreIngestor para registrar los embeddings en InMemoryEmbeddingStore.

5) Ajustar parámetros de búsqueda:
   - maxResults = 5.
   - minScore = 0.6.
   - Se llegó a estos valores con prueba y error, buscando equilibrio entre:
     - No devolver demasiados documentos irrelevantes.
     - No filtrar en exceso y perder contexto útil.

4. AGENTE DE BASE DE CONOCIMIENTO (KNOWLEDGEBASEAGENT)

Interfaz KnowledgeBaseAgent:

- Rol:
  - Describe cómo el modelo Claude debe interactuar con la base de conocimiento.
  - El agente:
    1. Analiza la solicitud del cliente.
    2. Busca información relevante en la base de conocimiento.
    3. Evalúa si la información encontrada resuelve el problema.
    4. Formula una respuesta clara y comprensible.

- SystemMessage (resumen funcional):
  - Eres un asistente especializado en búsqueda de soluciones en la base de conocimientos.
  - Debes buscar soluciones relevantes al problema descrito por el cliente.
  - Si las encuentras, preséntalas de forma clara y concisa.
  - Si no encuentras soluciones adecuadas, di que se enviará la solicitud a un operador humano.
  - Si encuentras soluciones, SIEMPRE debes:
    - Proporcionar una solución en el mensaje al cliente.
    - Preguntar si está satisfecho con la solución.
  - Si el cliente está satisfecho:
    - Indicar que el problema está resuelto y se puede cerrar la conversación.
  - Si el cliente no está satisfecho:
    - Indicar que se creará un ticket de soporte.
  - Responde únicamente con la solución encontrada (no añadir explicaciones adicionales fuera del objeto).
  - Detecta el idioma del cliente y responde en ese idioma (por defecto inglés si no lo reconoces).
  - Responde únicamente con el objeto KnowledgeBaseResult (sin texto adicional).

- UserMessage (plantilla de prompt):
  - Incluye variables:
    - text → mensaje del usuario.
    - customerCode → código del cliente.
    - selectedContractNumber → ID del contrato seleccionado.
    - issueType → tipo de problema.
  - Instrucción:
    - Proporcionar un objeto KnowledgeBaseResult con:
      - solutionFound: true si se encontró solución, false en caso contrario.
      - customerSatisfiedWithTheSolution: true si el cliente está satisfecho, false en caso contrario.
      - message: mensaje para mostrar al usuario (solución o aviso de creación de ticket).

- Output:
  - KnowledgeBaseResult:
    - solutionFound (boolean).
    - customerSatisfiedWithTheSolution (boolean).
    - message (string, texto para mostrar al cliente).

Punto clave:
- El modelo LLM no solo redacta la respuesta, sino que decide si la evidencia recuperada realmente resuelve el problema.
- La decisión es guiada por el prompt del SystemMessage, que pide ser honesto y conservador.

5. CUANDO LA KB NO ES SUFICIENTE: SISTEMA DE TICKETS

Si la base de conocimiento no logra resolver el problema, el flujo pasa a la gestión de tickets.

Componentes principales:

- TicketCreationAgent:
  - Interfaz que define cómo el modelo Claude debe generar la información para el ticket.
  - Debe producir:
    - Prioridad.
    - Descripción clara y concisa del problema.
    - Detalles necesarios para el equipo de soporte.
    - Mensaje para el usuario confirmando la creación del ticket (sin promesas de tiempos de resolución precisos).

- TicketCreationService:
  - Servicio que coordina la creación de tickets utilizando la información generada por TicketCreationAgent.
  - Aplica la lógica de negocio antes de llamar al sistema de tickets.

- CreateTicketService:
  - Servicio que efectivamente crea el ticket en el sistema de gestión de incidencias.
  - Devuelve un ID (UUID u otro) de referencia.

Determinación de prioridad:

- Niveles:
  - ALTA:
    - Problemas bloqueantes que impiden el uso del servicio o producto.
  - MEDIA:
    - Problemas significativos que limitan funcionalidades, pero permiten uso parcial.
  - BAJA:
    - Problemas menores, mejoras o consultas generales.

TicketCreationAgent – SystemMessage (resumen):

- Eres un asistente especializado en creación de tickets.
- Tu tarea es crear tickets para problemas que no se pueden resolver con la base de conocimiento.
- Debes determinar:
  1. Prioridad del ticket (ALTA, MEDIA, BAJA).
  2. Descripción clara y concisa del problema.
  3. Detalles necesarios para soporte.
  4. Mensaje para el cliente confirmando la creación del ticket y próximos pasos.
  5. No incluir información sobre tiempos concretos de resolución.
  6. Incluir una breve descripción del problema en el mensaje al cliente.
- Las prioridades se definen como:
  - ALTA: bloqueo total del servicio.
  - MEDIA: limitación parcial importante.
  - BAJA: problemas menores, mejoras, preguntas.
- Debes:
  - No incluir el ID del ticket en el mensaje al cliente.
  - Responder únicamente con el objeto TicketCreationResult.
  - No añadir explicaciones adicionales fuera del objeto.

UserMessage (plantilla):

- Incluir:
  - text → mensaje del usuario.
  - customerCode → código del cliente.
  - selectedContractNumber → número de contrato.
  - issueType → tipo de incidencia.
- Instrucción:
  - Crear un ticket de soporte con prioridad adecuada.
  - Proporcionar TicketCreationResult con:
    - priority: "ALTA", "MEDIA" o "BAJA".
    - description: descripción clara y detallada del problema.
    - message: mensaje para mostrar al usuario confirmando la creación del ticket y siguientes pasos.

Clase Ticket (resumen):

- Contiene campos:
  - customerId.
  - contractNumber.
  - issueType.
  - description.
  - priority.
- Se persiste mediante un TicketRepository.
- CreateTicketService:
  - Construye Ticket con builder.
  - Lo guarda en ticketRepository.
  - Devuelve ticket.getId().

6. FLUJO COMPLETO DESDE LA CONSULTA HASTA LA RESOLUCIÓN

Fase 1 – Búsqueda en la base de conocimientos:

1) El cliente envía un mensaje describiendo el problema.
2) KnowledgeBaseSearchService:
   - Analiza la solicitud.
   - Identifica conceptos clave.
   - Usa ContentRetriever para buscar documentos relevantes (via embeddings).
   - Evalúa si esos documentos contienen una solución suficiente.

3) Devuelve un KnowledgeBaseResult:
   - solutionFound indica si hay solución.
   - message contiene la solución o una indicación de que se generará ticket.

Fase 2 – Creación de ticket (si hace falta):

Condiciones para pasar a ticketing:
- solutionFound = false.
- O el cliente no está satisfecho con la solución propuesta (customerSatisfiedWithTheSolution = false).

En este caso:

1) TicketCreationService:
   - Vuelve a analizar la solicitud del cliente.
   - Determina prioridad (ALTA, MEDIA, BAJA).
   - Genera descripción detallada.
   - Crea el ticket en el sistema de tickets a través de CreateTicketService.

2) CreateTicketService:
   - Usa TicketRepository para guardar.
   - Devuelve un ID de ticket.

3) El asistente:
   - Devuelve un mensaje al cliente:
     - Confirmando la creación del ticket.
     - Explicando siguiente pasos.
     - Sin prometer SLA concretos y sin incluir el ID en el mensaje, según el prompt descrito.

Fase 3 – Comunicación con el cliente:

Escenarios:

- Si se encontró solución en la KB:
  - El asistente presenta la solución.
  - Pregunta si el cliente está satisfecho.
  - Si sí:
    - Marca el problema como resuelto.
  - Si no:
    - Desencadena creación de ticket.

- Si no se encontró solución:
  - El asistente explica que no hay solución inmediata.
  - Inicia proceso de creación de ticket.
  - Explica al cliente que un operador humano seguirá el caso.

Diseño de la comunicación:
- Transparente y tranquilizadora.
- El cliente debe sentirse acompañado aunque la KB no tenga la respuesta.

7. DESAFÍOS Y LECCIONES

El artículo destaca varios retos:

- Ajuste de parámetros de búsqueda:
  - maxResults y minScore tienen efecto directo en relevancia y recall.
  - Valores muy bajos de minScore → resultados irrelevantes.
  - Valores muy altos de minScore → se pierden documentos potencialmente útiles.

- Gestión de respuestas ambiguas:
  - Claude puede recuperar documentos, pero no estar seguro de si contienen la solución completa.
  - Se ajustó el SystemMessage para que el modelo sea conservador:
    - Ante duda, preferir crear ticket antes que dar solución parcial o errónea.

- Enriquecimiento de la KB:
  - Inicialmente la KB es limitada.
  - Se implementa un proceso de:
    - Usar tickets resueltos como nuevos documentos.
    - Alimentar la KB de manera continua.
  - Esto mejora progresivamente la capacidad de resolución autónoma.

- Métricas para mejora continua:
  - Tasa de resolución autónoma:
    - % de problemas resueltos sin ticket.
  - Precisión de la KB:
    - Relevancia de documentos recuperados vs. la consulta.
  - Precisión de prioridad:
    - Comparación entre prioridad asignada automáticamente y la confirmada por operadores.

8. BUENAS PRÁCTICAS A EXTRAER

- Diseñar la arquitectura para ser modular:
  - Separar:
    - RAG / KB.
    - Orquestador de lógica de negocio.
    - Sistema de ticketing.
  - Permite reemplazar:
    - EmbeddingStore in-memory → vector DB en producción.
    - Sistema de tickets (Redmine/Jira/GitHub) vía patrón adaptador.

- Usar outputs estructurados:
  - KnowledgeBaseResult para RAG.
  - TicketCreationResult para ticketing.
  - No permitir que el LLM devuelva texto libre mezclado con estructura.

- Política conservadora:
  - Mejor escalar a ticket ante incertidumbre que “inventar” una solución.

- Feedback loop entre tickets y KB:
  - Cada ticket resuelto genera material para nuevos documentos.
  - El sistema se vuelve más inteligente con el tiempo.

9. RESUMEN PARA IMPLEMENTACIÓN

Para implementar esta lógica en tu propio proyecto:

1) Configurar tokenizer, embedding model y EmbeddingStore.
2) Implementar pipeline de ingestión:
   - Documentos → splitter → embeddings → EmbeddingStore.
3) Configurar EmbeddingStoreContentRetriever:
   - maxResults = 5.
   - minScore = 0.6 (o ajustado a tu dominio).
4) Configurar DefaultRetrievalAugmentor + DefaultContentInjector.
5) Implementar KnowledgeBaseAgent:
   - Prompt de system y user similar al del artículo.
   - Output estructurado KnowledgeBaseResult.
6) Implementar KnowledgeBaseSearchService:
   - Llame al pipeline RAG.
   - Retorne KnowledgeBaseResult.
7) Implementar TicketCreationAgent:
   - Prompt para prioridad, descripción, mensaje al cliente.
   - Output TicketCreationResult.
8) Implementar TicketCreationService:
   - Lógica de negocio para ticketing.
9) Implementar CreateTicketService:
   - Integrar con sistema de tickets (persistencia o API).
10) Añadir métricas:
   - Autoresolución, precisión de KB, precisión de prioridad.
11) Implementar un proceso que convierta tickets resueltos en nuevos documentos para la KB.

Este manual resume y estructura la lógica RAG y de ticketing del artículo, listo para que un desarrollador la implemente o la adapte a otra stack.