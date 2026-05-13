import { json } from '@sveltejs/kit';

const LOCAL_API_BASE = 'http://localhost:11435/v1';

function isMockLikeResponse(payload) {
  const content = payload?.choices?.[0]?.message?.content;
  if (typeof content !== 'string') return false;
  return /^\s*gemma3\s+responder/i.test(content.trim());
}

export async function POST({ request, fetch }) {
  try {
    const body = await request.json();
    const model = body?.model || 'gemma3';
    const messages = Array.isArray(body?.messages) ? body.messages : [];
    const temperature = typeof body?.temperature === 'number' ? body.temperature : 0.6;
    const maxTokens = typeof body?.max_tokens === 'number' ? body.max_tokens : 500;
    const topP = typeof body?.top_p === 'number' ? body.top_p : 0.9;

    const upstream = await fetch(`${LOCAL_API_BASE}/chat/completions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model,
        messages,
        temperature,
        max_tokens: maxTokens,
        top_p: topP,
      }),
    });

    const text = await upstream.text();
    let payload;
    try {
      payload = JSON.parse(text);
    } catch {
      payload = { raw: text };
    }

    if (!upstream.ok) {
      return json(
        {
          error: `UPSTREAM_${upstream.status}`,
          details: payload,
        },
        { status: upstream.status }
      );
    }

    if (isMockLikeResponse(payload)) {
      return json(
        {
          error: 'DOCKER_MOCK_MODE',
          details: 'Upstream endpoint is returning simulated canned responses, not real inference.',
          payload,
        },
        { status: 502 }
      );
    }

    return json(payload);
  } catch (error) {
    return json(
      {
        error: 'LOCAL_PROXY_ERROR',
        details: String(error?.message || error),
      },
      { status: 500 }
    );
  }
}
