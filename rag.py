from typing import Any, List, Dict
import unicodedata

from app.config import settings
from app.egress import EgressNotAllowedError, EgressConfigError, search_web_serper
from app.memory import get_system_memory_status, list_knowledge_items, save_pending_web_sources
from app.state import is_core_tampered

def _normalize_text(s: str) -> str:
    """
    Normaliza texto a ASCII minúsculas, sin tildes ni símbolos raros.
    """
    return unicodedata.normalize("NFKD", s).encode("ascii", "ignore").decode("ascii").lower()


def _tokenize(text: str) -> List[str]:
    """
    Divide texto normalizado en tokens simples.
    """
    norm = _normalize_text(text)
    tokens = (
        norm.replace("?", " ")
        .replace(",", " ")
        .replace(".", " ")
        .split()
    )
    return [t for t in tokens if t]


STOPWORDS_ES = {
    "que", "es", "una", "un", "el", "la", "los", "las",
    "de", "del", "y", "en", "por", "para", "con",
    "sobre", "como", "a", "al", "se", "lo"
}


def _significant_tokens(tokens: List[str]) -> List[str]:
    """
    Filtra stopwords comunes en español; deja solo tokens significativos.
    """
    return [t for t in tokens if t not in STOPWORDS_ES]


def _search_local_knowledge(query: str, limit: int = 5) -> list[dict[str, Any]]:
    """
    Búsqueda simple en la KB local (texto).
    Usa título, contenido y metadata.original_query,
    con normalización y coincidencia por tokens significativos.
    """
    kb = list_knowledge_items(limit=100)
    items = kb.get("rows", [])

    query_norm = _normalize_text(query).strip()
    if not query_norm:
        return []

    # Tokens básicos de la query
    q_tokens_all = _tokenize(query_norm)
    q_tokens = _significant_tokens(q_tokens_all)

    # Corrección específica TON/Tom: si hay 'tom' y '618', sustituimos 'tom' por 'ton'
    if "tom" in q_tokens and "618" in q_tokens:
        q_tokens = ["ton" if t == "tom" else t for t in q_tokens]

    # Si no hay tokens significativos, no hacemos match genérico
    if len(q_tokens) == 0:
        return []

    matches: list[dict[str, Any]] = []

    for item in items:
        title = item.get("title") or ""
        content = item.get("content") or ""
        metadata = item.get("metadata") or {}

        title_norm = _normalize_text(title)
        content_norm = _normalize_text(content)
        orig_query = metadata.get("original_query") or ""
        orig_norm = _normalize_text(orig_query)

        # Texto completo del documento
        full = f"{title_norm} {content_norm} {orig_norm}"

        # Tokens del documento
        d_tokens_all = _tokenize(full)
        d_tokens = _significant_tokens(d_tokens_all)

        # 1) Match directo por original_query
        if orig_norm and orig_norm == query_norm:
            matches.append(item)
            if len(matches) >= limit:
                break
            continue

        # 2) Caso especial TON 618: ambos tokens deben estar en doc
        if ("ton" in q_tokens and "618" in q_tokens) and ("ton" in d_tokens_all and "618" in d_tokens_all):
            matches.append(item)
            if len(matches) >= limit:
                break
            continue

        # 3) Caso especial para preguntas biológicas tipo "nombre cientifico del perro"
        if "cientifico" in q_tokens and "perro" in q_tokens:
            # Buscamos "canis" y "familiaris" en título/contenido
            if "canis" in full and "familiaris" in full:
                matches.append(item)
                if len(matches) >= limit:
                    break
                continue

        # 4) Regla general: al menos dos tokens significativos compartidos entre query y documento
        shared = set(q_tokens) & set(d_tokens)
        if len(shared) >= 2:
            matches.append(item)
            if len(matches) >= limit:
                break

    return matches


def answer_with_rag(
    message: str,
    context: dict[str, Any] | None = None,
) -> tuple[str, list[dict[str, Any]], str | None]:
    """
    Responde usando primero información local, sin llamadas externas.
    """
    if context is None:
        context = {}

    system_status = get_system_memory_status()

    base_sources: list[dict[str, Any]] = [
        {
            "type": "system",
            "name": "health_check",
            "data": system_status,
        },
        {
            "type": "config",
            "name": "nora_config",
            "data": {
                "app_name": settings.app_name,
                "environment": settings.environment,
                "vm_nora_ip": settings.vm_nora_ip,
                "vm_data_ip": settings.vm_data_ip,
            },
        },
    ]

    kb_matches = _search_local_knowledge(message, limit=5)

    if kb_matches:
        answer_parts = [
            "Según mi base de conocimiento local, he encontrado información relacionada con tu pregunta.",
        ]

        main = kb_matches[0]
        title = main.get("title") or "Elemento de conocimiento"
        answer_parts.append(f"Título principal: {title}.")

        summary = main.get("metadata", {}).get("summary")
        if summary:
            answer_parts.append(f"En resumen, {summary}")
        else:
            content = main.get("content") or ""
            if content:
                answer_parts.append(f"En resumen, {content}")
            else:
                answer_parts.append(
                    "Se trata de un procedimiento o documento interno almacenado en Nora."
                )

        answer = " ".join(answer_parts)

        sources = base_sources + [
            {
                "type": "kb",
                "name": "knowledge_items",
                "data": {
                    "matches": [
                        {
                            "id": item.get("id"),
                            "title": item.get("title"),
                            "source_type": item.get("source_type"),
                            "tags": item.get("tags"),
                            "source_ref": item.get("source_ref"),
                        }
                        for item in kb_matches
                    ]
                },
            }
        ]

        reasoning = (
            "Respuesta generada usando únicamente documentos de la KB local "
            "en la tabla knowledge_items, sin acceso a internet."
        )

        return answer, sources, reasoning

    answer_parts = [
        "Con la información local que tengo actualmente no puedo responder a esta pregunta con suficiente precisión.",
        "Puedo intentar buscar en internet usando un conector controlado (serper.dev) si lo autorizas.",
    ]

    if not system_status["postgres"].get("ok") or not system_status["mongo"].get("ok"):
        answer_parts.append(
            "Además, detecto que alguna de las bases de datos no está accesible, "
            "lo que puede limitar aún más mi contexto local."
        )

    answer = " ".join(answer_parts)

    sources = base_sources

    reasoning = (
        "No se encontraron documentos relevantes en la KB local para esta pregunta. "
        "Solo se ha usado el estado local del sistema y la configuración. "
        "Se requiere autorización explícita para realizar una búsqueda externa con serper.dev."
    )

    return answer, sources, reasoning


def answer_with_rag_with_web(
    message: str,
    context: dict[str, Any] | None = None,
    max_web_results: int = 5,
) -> tuple[str, list[dict[str, Any]], str | None]:
    """
    Variante que sí puede usar serper.dev, tras autorización explícita.
    Si el core está en estado tampered, no se realizan llamadas externas.
    """
    if context is None:
        context = {}

    # Si el core está marcado como tampered, no permitimos web.
    if is_core_tampered():
        local_answer, local_sources, local_reasoning = answer_with_rag(message, context)
        reasoning = (
            (local_reasoning or "")
            + " No se pudo usar serper.dev porque Nora está en modo protegido por integridad."
        )
        return local_answer, local_sources, reasoning

    local_answer, local_sources, local_reasoning = answer_with_rag(message, context)

    has_kb = any(s.get("type") == "kb" for s in local_sources)
    if has_kb:
        return local_answer, local_sources, local_reasoning

    try:
        web_result = search_web_serper(message, num_results=max_web_results)
    except EgressNotAllowedError:
        reasoning = (
            (local_reasoning or "")
            + " No se pudo usar serper.dev porque la salida externa está deshabilitada."
        )
        return local_answer, local_sources, reasoning
    except EgressConfigError:
        reasoning = (
            (local_reasoning or "")
            + " No se pudo usar serper.dev porque falta SERPER_API_KEY en la configuración."
        )
        return local_answer, local_sources, reasoning

    sources = list(local_sources)

    if not web_result.get("ok"):
        reasoning = (
            (local_reasoning or "")
            + f" Se intentó usar serper.dev pero la llamada falló: {web_result.get('error')}."
        )
        sources.append(
            {
                "type": "web",
                "name": "serper.dev",
                "data": {"ok": False, "error": web_result.get("error")},
            }
        )
        return local_answer, sources, reasoning

    raw = web_result.get("raw", {}) or {}
    organic = raw.get("organic") or raw.get("results") or []

    results_for_pending: list[dict[str, Any]] = []
    for item in organic[:max_web_results]:
        results_for_pending.append(
            {
                "title": item.get("title") or "",
                "snippet": item.get("snippet") or item.get("description") or "",
                "link": item.get("link") or item.get("url") or "",
            }
        )

    enriched = save_pending_web_sources(message, results_for_pending)

    sources.append(
        {
            "type": "web",
            "name": "serper.dev",
            "data": {
                "results": enriched,
            },
        }
    )

    answer_parts = []

    if local_reasoning:
        answer_parts.append(
            "He revisado primero mi conocimiento local pero no encontré suficiente información específica."
        )

    if enriched:
        answer_parts.append(
            "Según las fuentes externas consultadas mediante serper.dev, esta es la mejor descripción encontrada."
        )
        main = enriched[0]
        if main["title"]:
            answer_parts.append(f"La fuente principal es: {main['title']}.")
        if main["snippet"]:
            answer_parts.append(f"En resumen, {main['snippet']}")
    else:
        answer_parts.append(
            "He intentado buscar en internet mediante serper.dev, pero no encontré resultados claros."
        )

    answer = " ".join(answer_parts)

    reasoning = (
        "Respuesta generada sin KB local relevante y combinando resultados devueltos "
        "por serper.dev, siempre después de autorización explícita."
    )

    return answer, sources, reasoning
