// Theme data is same-origin and bounded before it reaches the Rust parser.
export async function fetchTheme(fetcher = fetch, timeoutMs = 10_000, controller = new AbortController()) {
  const timeout = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetcher('assets/macos27.json', {
      cache: 'no-store', credentials: 'same-origin', redirect: 'error', signal: controller.signal,
    });
    if (!response.ok) throw new Error(`Theme request failed (${response.status})`);
    if (!response.body) throw new Error('Theme response has no body');
    const reader = response.body.getReader();
    const decoder = new TextDecoder('utf-8', { fatal: true });
    const limit = 2 * 1024 * 1024;
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
      return text + decoder.decode();
    } catch (error) {
      await reader.cancel().catch(() => {});
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
