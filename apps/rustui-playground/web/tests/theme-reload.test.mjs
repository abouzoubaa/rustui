import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const source = await readFile(new URL('../theme-reload.js', import.meta.url));
const { fetchTheme, startThemeFetch } = await import(`data:text/javascript;base64,${source.toString('base64')}`);

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
