// Theme data is same-origin and bounded before it reaches the Rust parser.
export async function fetchTheme(fetcher = fetch, timeoutMs = 10_000, controller = new AbortController(), cache = null) {
  const timeout = setTimeout(() => controller.abort(), timeoutMs);
  try {
    let generation = cache ? (cache.generation = {}) : null;
    const previous = cache && cache.etag ? { etag: cache.etag, text: cache.text } : null;
    const options = {
      cache: cache ? 'reload' : 'no-store', credentials: 'same-origin', redirect: 'error', signal: controller.signal,
      ...(previous ? { headers: { 'If-None-Match': previous.etag } } : {}),
    };
    let response = await fetcher('assets/macos27.json', options);
    if (response.status === 304) {
      if (previous && response.headers.get('etag') === previous.etag &&
          !/\bno-store\b/i.test(response.headers.get('cache-control') || '')) {
        return previous.text;
      }
      if (cache && cache.generation === generation) {
        clearThemeCache(cache);
        generation = cache.generation;
      }
      response = await fetcher('assets/macos27.json', { ...options, headers: {} });
    }
    if (cache && cache.generation === generation && /\bno-store\b/i.test(response.headers.get('cache-control') || '')) {
      clearThemeCache(cache);
      generation = cache.generation;
    }
    if (!response.ok) {
      if (response.body) void response.body.cancel().catch(() => {});
      throw new Error(`Theme request failed (${response.status})`);
    }
    if (!response.body) throw new Error('Theme response has no body');
    const limit = 2 * 1024 * 1024;
    const length = response.headers.get('content-length');
    const encoding = response.headers.get('content-encoding');
    if ((!encoding || encoding.toLowerCase() === 'identity') &&
        length && /^\d+$/.test(length) && Number(length) > limit) {
      void response.body.cancel().catch(() => {});
      throw new Error('Theme response exceeds 2 MiB');
    }
    const reader = response.body.getReader();
    const decoder = new TextDecoder('utf-8', { fatal: true });
    let size = 0;
    let text = '';
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        size += value.byteLength;
        if (size > limit) throw new Error('Theme response exceeds 2 MiB');
        text += decoder.decode(value, { stream: true });
      }
      text += decoder.decode();
      if (cache && cache.generation === generation) {
        const etag = response.headers.get('etag');
        const noStore = /\bno-store\b/i.test(response.headers.get('cache-control') || '');
        if (!noStore && etag && /^"[\x21\x23-\x7e\x80-\xff]*"$/.test(etag)) {
          cache.etag = etag;
          cache.text = text;
        } else {
          clearThemeCache(cache);
        }
      }
      return text;
    } catch (error) {
      void reader.cancel().catch(() => {});
      throw error;
    } finally {
      reader.releaseLock();
    }
  } finally {
    clearTimeout(timeout);
  }
}

// The Rust task owns this handle and cancels it when the task is dropped.
export function startThemeFetch(fetcher = fetch, timeoutMs = 10_000) {
  const controller = new AbortController();
  return {
    promise: fetchTheme(fetcher, timeoutMs, controller),
    cancel() { controller.abort(); },
  };
}

export function createThemeCache() {
  return { etag: null, text: null, generation: {} };
}

export function clearThemeCache(cache) {
  cache.generation = {};
  cache.etag = null;
  cache.text = null;
}

export function startCachedThemeFetch(cache, fetcher = fetch, timeoutMs = 10_000) {
  const controller = new AbortController();
  return {
    promise: fetchTheme(fetcher, timeoutMs, controller, cache),
    cancel() { controller.abort(); },
  };
}
