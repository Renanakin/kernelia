import asyncio
import json
import os
import time
from typing import Any, AsyncIterator, Dict, List

import httpx
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, ConfigDict, Field

OLLAMA_BASE_URL = os.getenv("OLLAMA_BASE_URL", "http://localhost:11434")
OLLAMA_MODEL = os.getenv("OLLAMA_MODEL", "gemma3")
OLLAMA_TIMEOUT_SECONDS = float(os.getenv("OLLAMA_TIMEOUT_SECONDS", "120"))
OLLAMA_RETRIES = int(os.getenv("OLLAMA_RETRIES", "1"))
OLLAMA_KEEP_ALIVE = os.getenv("OLLAMA_KEEP_ALIVE", "5m")
OLLAMA_DEFAULT_TEMPERATURE = float(os.getenv("OLLAMA_DEFAULT_TEMPERATURE", "0.6"))
OLLAMA_DEFAULT_MAX_TOKENS = int(os.getenv("OLLAMA_DEFAULT_MAX_TOKENS", "500"))
OLLAMA_MAX_TOKENS_CAP = int(os.getenv("OLLAMA_MAX_TOKENS_CAP", "1200"))
OLLAMA_TOP_P = float(os.getenv("OLLAMA_TOP_P", "0.9"))
OLLAMA_NUM_THREAD = int(os.getenv("OLLAMA_NUM_THREAD", "6"))
OLLAMA_NUM_CTX = int(os.getenv("OLLAMA_NUM_CTX", "4096"))
OLLAMA_NUM_GPU = os.getenv("OLLAMA_NUM_GPU", "").strip()
OLLAMA_PARALLELISM = max(1, int(os.getenv("OLLAMA_PARALLELISM", "2")))

LLM_SEMAPHORE = asyncio.Semaphore(OLLAMA_PARALLELISM)

app = FastAPI(title="KernelIA Ollama OpenAI Proxy", version="2.0.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class ChatMessage(BaseModel):
    model_config = ConfigDict(extra="allow")

    role: str
    content: str | None = ""


class ChatCompletionRequest(BaseModel):
    model_config = ConfigDict(extra="allow")

    model: str | None = None
    messages: List[ChatMessage] = Field(default_factory=list)
    temperature: float | None = None
    max_tokens: int | None = None
    top_p: float | None = None
    stream: bool | None = False
    tools: Any | None = None
    tool_choice: Any | None = None


def _max_tokens(req: ChatCompletionRequest) -> int:
    requested = req.max_tokens if req.max_tokens is not None else OLLAMA_DEFAULT_MAX_TOKENS
    return max(1, min(int(requested), OLLAMA_MAX_TOKENS_CAP))


def _options(req: ChatCompletionRequest) -> Dict[str, Any]:
    options: Dict[str, Any] = {
        "temperature": req.temperature if req.temperature is not None else OLLAMA_DEFAULT_TEMPERATURE,
        "top_p": req.top_p if req.top_p is not None else OLLAMA_TOP_P,
        "num_predict": _max_tokens(req),
        "num_thread": OLLAMA_NUM_THREAD,
        "num_ctx": OLLAMA_NUM_CTX,
    }
    if OLLAMA_NUM_GPU:
        options["num_gpu"] = int(OLLAMA_NUM_GPU)
    return options


def _ollama_payload(req: ChatCompletionRequest, stream: bool) -> Dict[str, Any]:
    return {
        "model": req.model or OLLAMA_MODEL,
        "messages": [{"role": m.role, "content": m.content or ""} for m in req.messages],
        "stream": stream,
        "options": _options(req),
        "keep_alive": OLLAMA_KEEP_ALIVE,
    }


def _openai_chunk(model: str, content: str = "", finish_reason: str | None = None) -> str:
    payload = {
        "id": "chatcmpl-kernelia-ollama",
        "object": "chat.completion.chunk",
        "created": int(time.time()),
        "model": model,
        "choices": [
            {
                "index": 0,
                "delta": {"content": content} if content else {},
                "finish_reason": finish_reason,
            }
        ],
    }
    return f"data: {json.dumps(payload, ensure_ascii=False)}\n\n"


async def _post_ollama(payload: Dict[str, Any]) -> httpx.Response:
    last_exc: Exception | None = None
    for attempt in range(OLLAMA_RETRIES + 1):
        try:
            async with httpx.AsyncClient(timeout=OLLAMA_TIMEOUT_SECONDS) as client:
                return await client.post(f"{OLLAMA_BASE_URL}/api/chat", json=payload)
        except Exception as exc:
            last_exc = exc
            if attempt >= OLLAMA_RETRIES:
                raise HTTPException(status_code=502, detail=f"OLLAMA_CONNECTION_ERROR: {exc}") from exc
            await asyncio.sleep(0.6 * (attempt + 1))

    raise HTTPException(status_code=502, detail=f"OLLAMA_CONNECTION_ERROR: {last_exc}")


async def _stream_ollama(payload: Dict[str, Any], model: str) -> AsyncIterator[str]:
    async with LLM_SEMAPHORE:
        async with httpx.AsyncClient(timeout=OLLAMA_TIMEOUT_SECONDS) as client:
            async with client.stream("POST", f"{OLLAMA_BASE_URL}/api/chat", json=payload) as response:
                if response.status_code >= 400:
                    body = await response.aread()
                    raise HTTPException(status_code=response.status_code, detail=f"OLLAMA_ERROR: {body.decode(errors='replace')}")

                async for line in response.aiter_lines():
                    if not line:
                        continue
                    try:
                        item = json.loads(line)
                    except json.JSONDecodeError:
                        continue

                    content = ((item.get("message") or {}).get("content") or "")
                    if content:
                        yield _openai_chunk(model, content=content)

                    if item.get("done"):
                        yield _openai_chunk(model, finish_reason="stop")
                        yield "data: [DONE]\n\n"
                        break


@app.get("/health")
async def health() -> Dict[str, Any]:
    return {
        "status": "ok",
        "backend": "ollama",
        "ollama_base_url": OLLAMA_BASE_URL,
        "default_model": OLLAMA_MODEL,
        "settings": {
            "timeout_seconds": OLLAMA_TIMEOUT_SECONDS,
            "parallelism": OLLAMA_PARALLELISM,
            "temperature": OLLAMA_DEFAULT_TEMPERATURE,
            "max_tokens": OLLAMA_DEFAULT_MAX_TOKENS,
            "max_tokens_cap": OLLAMA_MAX_TOKENS_CAP,
            "top_p": OLLAMA_TOP_P,
            "num_thread": OLLAMA_NUM_THREAD,
            "num_ctx": OLLAMA_NUM_CTX,
            "keep_alive": OLLAMA_KEEP_ALIVE,
        },
    }


@app.get("/v1/models")
async def models() -> Dict[str, Any]:
    return {
        "object": "list",
        "data": [
            {
                "id": OLLAMA_MODEL,
                "object": "model",
                "owned_by": "ollama",
            }
        ],
    }


@app.post("/v1/chat/completions", response_model=None)
async def chat_completions(req: ChatCompletionRequest):
    model = req.model or OLLAMA_MODEL
    if not req.messages:
        raise HTTPException(status_code=400, detail="messages is required")

    payload = _ollama_payload(req, stream=bool(req.stream))

    if req.stream:
        return StreamingResponse(
            _stream_ollama(payload, model),
            media_type="text/event-stream",
            headers={
                "Cache-Control": "no-cache",
                "Connection": "keep-alive",
                "X-Accel-Buffering": "no",
            },
        )

    async with LLM_SEMAPHORE:
        response = await _post_ollama(payload)

    if response.status_code >= 400:
        raise HTTPException(status_code=response.status_code, detail=f"OLLAMA_ERROR: {response.text}")

    data = response.json()
    content = ((data.get("message") or {}).get("content") or "").strip()
    if not content:
        raise HTTPException(status_code=502, detail="OLLAMA_EMPTY_RESPONSE")

    return {
        "id": "chatcmpl-kernelia-ollama",
        "object": "chat.completion",
        "created": int(time.time()),
        "model": model,
        "choices": [
            {
                "index": 0,
                "message": {"role": "assistant", "content": content},
                "finish_reason": "stop",
            }
        ],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0,
        },
    }
