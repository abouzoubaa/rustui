// Consumer-owned client for the same-origin canonical Playground iframe.
export const PROTOCOL = 'rustui.preview.postmessage';
export const VERSION = 1;
export const CONSUMER = 'rustui.playground.browser-preview-host';
export const MAX_BYTES = 1048576;
const token = value => typeof value === 'string' && /^[0-9a-f]{32}$/.test(value);
const bounded = value => typeof value === 'string' && value.length <= MAX_BYTES && new TextEncoder().encode(value).byteLength <= MAX_BYTES;
export class ReferenceClient {
  constructor(frame, profile, browser = window, timeout = 15000) {
    if (!['preview', 'composition'].includes(profile) || browser.location.origin === 'null') throw new Error('invalid host');
    this.frame = frame; this.profile = profile; this.browser = browser; this.origin = browser.location.origin;
    this.target = frame.contentWindow; this.timeout = timeout; this.session = null; this.stamp = null; this.sequence = 1n;
    this.serial = 0; this.pending = null; this.alive = true; this.loaded = false; this.readySeen = false;
    const bytes = new Uint8Array(16); browser.crypto.getRandomValues(bytes);
    this.nonce = Array.from(bytes, n => n.toString(16).padStart(2, '0')).join('');
    this.ready = new Promise((resolve, reject) => { this.readyResolve = resolve; this.readyReject = reject; });
    this.readyTimer = browser.setTimeout(() => this.dispose('ready timeout'), timeout);
    this.message = event => this.receive(event);
    this.pagehide = () => this.dispose('page closed');
    this.load = () => { if (this.loaded) this.dispose('frame reloaded'); this.loaded = true; };
    browser.addEventListener('message', this.message); browser.addEventListener('pagehide', this.pagehide);
    frame.addEventListener('load', this.load);
    if (browser.MutationObserver) {
      this.observer = new browser.MutationObserver(() => { if (!frame.isConnected) this.dispose('frame removed'); });
      this.observer.observe(browser.document, { childList: true, subtree: true });
    }
  }
  receive(event) {
    if (!this.alive || event.source !== this.target || event.origin !== this.origin || !bounded(event.data)) return;
    let reply; try { reply = JSON.parse(event.data); } catch { return; }
    if (!reply || reply.protocol !== PROTOCOL || reply.version !== VERSION) return;
    if (reply.event === 'ready') {
      if (reply.profile !== this.profile || this.readySeen) return;
      this.readySeen = true; this.browser.clearTimeout(this.readyTimer); this.readyResolve(); return;
    }
    if (reply.event === 'closed') {
      if (this.session && reply.session === this.session) this.dispose('remote closed');
      return;
    }
    const pending = this.pending;
    if (!pending || reply.request !== pending.request || reply.sequence !== pending.sequence) return;
    if (pending.connect) {
      if (reply.error !== null || !token(reply.session) || reply.result?.nonce !== this.nonce || reply.result?.profile !== this.profile) {
        this.dispose('invalid handshake'); return;
      }
      this.session = reply.session;
    } else if (reply.session !== this.session) return;
    if ((reply.error !== null && typeof reply.error !== 'string') || (reply.error === null && (reply.result === null || typeof reply.result !== 'object'))) return;
    this.browser.clearTimeout(pending.timer); this.pending = null;
    if (reply.error === null && Object.hasOwn(reply.result, 'stamp')) this.stamp = reply.result.stamp;
    if (reply.error !== null) { const error = new Error(reply.error); error.code = reply.error; pending.reject(error); }
    else pending.resolve(reply.result);
  }
  async connect() {
    await this.ready;
    if (this.session) throw new Error('already connected');
    return this.send({ kind: 'connect', nonce: this.nonce }, true);
  }
  send(command, connect = false) {
    if (!this.alive || this.pending || (!connect && !this.session)) return Promise.reject(new Error('client unavailable'));
    const request = `request-${++this.serial}`, sequence = connect ? '0' : this.sequence.toString();
    const wire = JSON.stringify({ protocol: PROTOCOL, version: VERSION, client: CONSUMER, request, session: connect ? null : this.session, sequence, command });
    if (!bounded(wire)) return Promise.reject(new Error('message_too_large'));
    if (!connect) this.sequence += 1n;
    return new Promise((resolve, reject) => {
      const timer = this.browser.setTimeout(() => this.dispose('reply timeout'), this.timeout);
      this.pending = { request, sequence, connect, resolve, reject, timer };
      try { this.target.postMessage(wire, this.origin); } catch { this.dispose('postMessage failed'); }
    });
  }
  async disconnect(stamp = this.stamp) {
    const reply = await this.send({ kind: 'disconnect', stamp });
    this.dispose('disconnected'); return reply;
  }
  dispose(reason = 'disposed') {
    if (!this.alive) return;
    this.alive = false; this.browser.clearTimeout(this.readyTimer);
    this.browser.removeEventListener('message', this.message); this.browser.removeEventListener('pagehide', this.pagehide);
    this.frame.removeEventListener('load', this.load); this.observer?.disconnect();
    this.readyReject(new Error(reason));
    if (this.pending) { this.browser.clearTimeout(this.pending.timer); this.pending.reject(new Error(reason)); this.pending = null; }
    this.session = null; this.stamp = null;
  }
}
