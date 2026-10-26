import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { webcrypto } from 'node:crypto';
const load = async name => import(`data:text/javascript;base64,${Buffer.from(await readFile(new URL(`../${name}`, import.meta.url))).toString('base64')}`);
const bridge = await load('preview-transport.js');
const { ReferenceClient, PROTOCOL, VERSION } = await load('reference-client.js');
function surface() {
  const listeners = new Map(), sent = [], origin = 'http://localhost:8080';
  const value = { sent, location: { origin, search: '?rustuiPreviewHost=preview' }, crypto: webcrypto,
    setTimeout, clearTimeout,
    addEventListener(name, fn) { if (!listeners.has(name)) listeners.set(name, new Set()); listeners.get(name).add(fn); },
    removeEventListener(name, fn) { listeners.get(name)?.delete(fn); },
    emit(name, event) { for (const fn of [...(listeners.get(name) || [])]) fn(event); },
    count(name) { return listeners.get(name)?.size || 0; },
    postMessage(data, target) { sent.push({ data, target }); },
  };
  return value;
}
function setupClient() {
  const browser = surface(), child = surface(), frame = surface(); frame.contentWindow = child; frame.isConnected = true;
  const client = new ReferenceClient(frame, 'preview', browser, 1000);
  const receive = value => browser.emit('message', { source: child, origin: browser.location.origin, data: JSON.stringify({ protocol: PROTOCOL, version: VERSION, ...value }) });
  return { browser, child, frame, client, receive };
}
function respond(env, result, overrides = {}) {
  const request = JSON.parse(env.child.sent.at(-1).data);
  env.receive({ request: request.request, sequence: request.sequence, session: env.client.session || 'a'.repeat(32), result, error: null, ...overrides });
}
async function connected(env) {
  env.receive({ event: 'ready', profile: 'preview' });
  const promise = env.client.connect(); await Promise.resolve();
  respond(env, { nonce: env.client.nonce, profile: 'preview' }); await promise;
}
test('actual child bridge rejects foreign source/origin, object and UTF8 overflow; closes listeners once', () => {
  const browser = surface(); browser.parent = surface(); const received = [];
  let closed = 0;
  const handle = bridge.createBridge(value => received.push(value), () => { closed += 1; }, browser);
  const message = data => ({ source: browser.parent, origin: browser.location.origin, data });
  browser.emit('message', { ...message('ok'), origin: 'https://other.example' });
  browser.emit('message', { ...message('ok'), source: {} });
  browser.emit('message', message({ command: 'bad' }));
  browser.emit('message', message('é'.repeat(524289)));
  browser.emit('message', message('x'.repeat(1048577)));
  browser.emit('message', message('accepted')); assert.deepEqual(received, ['accepted']);
  bridge.markReady(handle); bridge.markReady(handle); assert.equal(browser.parent.sent.length, 1);
  bridge.postReply(handle, 'reply', 'a'.repeat(32)); assert.equal(browser.parent.sent.at(-1).target, browser.location.origin);
  bridge.postReply(handle, 'error from stale session', '');
  browser.emit('pagehide', {}); bridge.closeBridge(handle);
  assert.equal(JSON.parse(browser.parent.sent.at(-1).data).session, 'a'.repeat(32));
  assert.equal(browser.count('message'), 0); assert.equal(browser.count('pagehide'), 0); assert.equal(closed, 1);
  assert.equal(browser.parent.sent.filter(value => value.data.includes('"closed"')).length, 1);
});
test('actual bridge profile requires same-origin non-top-level iframe', () => {
  const browser = surface(); browser.parent = browser; assert.equal(bridge.requestedProfile(browser), '');
  browser.parent = surface(); browser.parent.location.origin = 'https://other.example'; assert.equal(bridge.requestedProfile(browser), '');
  browser.parent.location.origin = browser.location.origin; assert.equal(bridge.requestedProfile(browser), 'preview');
  assert.match(bridge.freshToken(browser), /^[0-9a-f]{32}$/);
});
test('actual client pins source/origin/request/session/sequence and preserves raw numeric source', async () => {
  const env = setupClient(); await connected(env);
  const source = '{"large":9223372036854775807}'; const promise = env.client.send({ kind: 'mount', source });
  assert.equal(JSON.parse(env.child.sent.at(-1).data).command.source, source);
  const request = JSON.parse(env.child.sent.at(-1).data), reply = { protocol: PROTOCOL, version: VERSION, request: request.request, sequence: request.sequence, session: env.client.session, result: { stamp: null }, error: null };
  env.browser.emit('message', { source: {}, origin: env.browser.location.origin, data: JSON.stringify(reply) });
  env.browser.emit('message', { source: env.child, origin: 'https://other.example', data: JSON.stringify(reply) });
  env.receive({ ...reply, session: 'b'.repeat(32) }); env.receive({ ...reply, request: 'stale' }); env.receive({ ...reply, sequence: '99' });
  assert.ok(env.client.pending); respond(env, { stamp: null }); assert.deepEqual(await promise, { stamp: null });
  const error = env.client.send({ kind: 'export', stamp: null }); respond(env, null, { error: 'stale_stamp' }); await assert.rejects(error, { code: 'stale_stamp' });
  assert.equal(env.client.sequence, 3n); env.client.dispose();
});
test('client disconnect waits reply and removes all listeners; stale closed event ignored', async () => {
  const env = setupClient(); await connected(env);
  env.receive({ event: 'closed', session: 'b'.repeat(32) }); assert.equal(env.client.alive, true);
  const pending = env.client.disconnect(); assert.equal(env.client.alive, true);
  respond(env, { disconnected: true }); await pending;
  assert.equal(env.client.alive, false); assert.equal(env.browser.count('message'), 0); assert.equal(env.frame.count('load'), 0);
});
test('client rejects concurrent requests and disposes pending request on reload', async () => {
  const env = setupClient(); await connected(env);
  const pending = env.client.send({ kind: 'catalog' });
  await assert.rejects(env.client.send({ kind: 'catalog' }), /unavailable/);
  env.frame.emit('load', {}); env.frame.emit('load', {}); await assert.rejects(pending, /reloaded/);
  assert.equal(env.client.alive, false);
});
test('client validates echoed handshake nonce and version', async () => {
  const env = setupClient(); env.receive({ event: 'ready', profile: 'preview', version: 2 }); assert.equal(env.client.readySeen, false);
  env.receive({ event: 'ready', profile: 'preview' }); const promise = env.client.connect(); await Promise.resolve();
  respond(env, { nonce: 'b'.repeat(32), profile: 'preview' }); await assert.rejects(promise, /handshake/);
});
