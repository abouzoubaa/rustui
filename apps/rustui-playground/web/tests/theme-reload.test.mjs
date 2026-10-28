import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const source = await readFile(new URL('../theme-reload.js', import.meta.url));
const { fetchTheme, startThemeFetch, createThemeCache, clearThemeCache, startCachedThemeFetch } = await import(`data:text/javascript;base64,${source.toString('base64')}`);

test('fetches uncached same-origin UTF-8 theme data', async () => {
  const result = await fetchTheme(async (url, options) => {
    assert.equal(url, 'assets/macos27.json');
    assert.equal(options.cache, 'no-store');
    assert.equal(options.credentials, 'same-origin');
    assert.equal(options.redirect, 'error');
    assert.ok(options.signal instanceof AbortSignal);
    return new Response('{"name":"عربي"}');
  });
  assert.equal(result, '{"name":"عربي"}');
});

test('rejects failed, oversized, and invalid UTF-8 responses', async () => {
  await assert.rejects(fetchTheme(async () => new Response('', { status: 404 })), /404/);
  await assert.rejects(fetchTheme(async () => new Response(new Uint8Array(2 * 1024 * 1024 + 1))), /exceeds/);
  await assert.rejects(fetchTheme(async () => new Response(new Uint8Array([0xff]))), /encoded data/);
  assert.equal((await fetchTheme(async () => new Response(new Uint8Array(2 * 1024 * 1024)))).length, 2 * 1024 * 1024);
});

test('rejects a declared oversized uncompressed body without reading it', async () => {
  let cancelled = false;
  const body = new ReadableStream({ cancel() { cancelled = true; } });
  await assert.rejects(fetchTheme(async () => new Response(body, {
    headers: { 'content-length': String(2 * 1024 * 1024 + 1) },
  })), /exceeds 2 MiB/);
  assert.equal(cancelled, true);

  // Content-Length describes compressed bytes, while the stream is decoded.
  assert.equal(await fetchTheme(async () => new Response('ok', {
    headers: { 'content-length': String(2 * 1024 * 1024 + 1), 'content-encoding': 'gzip' },
  })), 'ok');
});

test('revalidates strong ETags and falls back when validation is unavailable', async () => {
  const cache = createThemeCache();
  let calls = 0;
  const fetcher = async (_url, options) => {
    calls++;
    assert.equal(options.cache, 'reload');
    if (calls === 1) return new Response('first', { headers: { etag: '"one"' } });
    if (calls === 2) {
      assert.equal(options.headers['If-None-Match'], '"one"');
      return new Response(null, { status: 304, headers: { etag: '"one"' } });
    }
    if (calls === 3) return new Response(null, { status: 304 });
    assert.deepEqual(options.headers, {});
    return new Response('second', { headers: { etag: 'W/"two"' } });
  };
  assert.equal(await startCachedThemeFetch(cache, fetcher).promise, 'first');
  assert.equal(await startCachedThemeFetch(cache, fetcher).promise, 'first');
  assert.equal(await startCachedThemeFetch(cache, fetcher).promise, 'second');
  assert.equal(cache.etag, null);
  clearThemeCache(cache);
  assert.equal(cache.text, null);
});

test('does not retain a response that forbids storage', async () => {
  const cache = createThemeCache();
  await startCachedThemeFetch(cache, async () => new Response('private', {
    headers: { etag: '"one"', 'cache-control': 'no-store' },
  })).promise;
  assert.equal(cache.etag, null);
  assert.equal(cache.text, null);
});

test('clears the cached body when a 304 forbids storage even if retry fails', async () => {
  const cache = createThemeCache();
  await startCachedThemeFetch(cache, async () => new Response('old', {
    headers: { etag: '"old"' },
  })).promise;
  let calls = 0;
  await assert.rejects(startCachedThemeFetch(cache, async () => {
    if (calls++ === 0) return new Response(null, {
      status: 304, headers: { etag: '"old"', 'cache-control': 'no-store' },
    });
    throw new Error('retry failed');
  }).promise, /retry failed/);
  assert.equal(cache.etag, null);
  assert.equal(cache.text, null);
});

test('clears the cached body before rejecting a no-store oversized response', async () => {
  const cache = createThemeCache();
  await startCachedThemeFetch(cache, async () => new Response('old', {
    headers: { etag: '"old"' },
  })).promise;
  await assert.rejects(startCachedThemeFetch(cache, async () => new Response('large', {
    headers: { 'content-length': String(2 * 1024 * 1024 + 1), 'cache-control': 'no-store' },
  })).promise, /exceeds 2 MiB/);
  assert.equal(cache.etag, null);
  assert.equal(cache.text, null);
});

test('aborts a request that does not complete', async () => {
  await assert.rejects(fetchTheme((_url, { signal }) => new Promise((_resolve, reject) => {
    signal.addEventListener('abort', () => reject(new Error('timed out')), { once: true });
  }), 5), /timed out/);
});

test('owner cancellation aborts a pending request immediately', async () => {
  let signal;
  const request = startThemeFetch((_url, options) => {
    signal = options.signal;
    return new Promise((_resolve, reject) => {
      signal.addEventListener('abort', () => reject(new Error('canceled')), { once: true });
    });
  });
  assert.equal(signal.aborted, false);
  request.cancel();
  assert.equal(signal.aborted, true);
  await assert.rejects(request.promise, /canceled/);
  request.cancel();
});

test('decodes split UTF-8 and cancels while the response body is stalled', async () => {
  const bytes = new TextEncoder().encode('عربي');
  const chunks = new ReadableStream({
    start(controller) {
      for (const byte of bytes) controller.enqueue(Uint8Array.of(byte));
      controller.close();
    },
  });
  assert.equal(await fetchTheme(async () => new Response(chunks)), 'عربي');

  let started;
  const reading = new Promise(resolve => { started = resolve; });
  const request = startThemeFetch(async (_url, { signal }) => new Response(new ReadableStream({
    start(controller) {
      controller.enqueue(Uint8Array.of(65));
      signal.addEventListener('abort', () => controller.error(new Error('body canceled')), { once: true });
    },
    pull() { started(); },
  })));
  await reading;
  request.cancel();
  await assert.rejects(request.promise, /body canceled/);
});
