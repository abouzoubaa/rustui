import { ReferenceClient } from './reference-client.js';
const $ = id => document.getElementById(id);
let client, stamp = null, catalog, capabilities;
const controls = ['disconnect', 'roundtrip', 'catalog', 'draft', 'mount', 'export', 'edit', 'signal'];
const log = value => { $('results').textContent = `${typeof value === 'string' ? value : JSON.stringify(value, null, 2)}\n${$('results').textContent}`.slice(0, 30000); };
const status = value => { $('status').textContent = value; };
function enabled(value) { for (const id of controls) $(id).disabled = !value; }
async function request(command) {
  const result = await client.send(command);
  if (Object.hasOwn(result, 'stamp')) stamp = result.stamp;
  status(`Accepted: ${command.kind.replaceAll('_', ' ')}`);
  log(command.kind === 'catalog' ? {catalog_bytes: new TextEncoder().encode(result.source).byteLength} : result); return result;
}
async function refresh() {
  capabilities = await request({ kind: 'capabilities' });
  $('capabilities').textContent = JSON.stringify(capabilities, null, 2);
  const result = await request({ kind: 'catalog' });
  $('catalog-source').textContent = result.source;
  // Only IDs and schema versions are used here. Numeric property values stay in source text.
  catalog = JSON.parse(result.source);
  $('component').replaceChildren(...Object.values(catalog.components).map(schema => {
    const option = document.createElement('option'); option.value = schema.id;
    option.textContent = schema.id; return option;
  }));
}
async function connect() {
  enabled(false); client?.dispose('reconnecting'); stamp = null;
  const old = $('preview'), frame = document.createElement('iframe');
  frame.id = 'preview'; frame.title = old.title; old.replaceWith(frame);
  client = new ReferenceClient(frame, $('profile').value);
  frame.src = `./index.html?rustuiPreviewHost=${encodeURIComponent($('profile').value)}`;
  status('Loading canonical iframe…');
  capabilities = await client.connect(); await refresh(); enabled(true); status(`Connected: ${capabilities.profile}`);
}
function preview(component, properties = {}, actions = {}, id = 'browser-reference') {
  const schema = catalog.components[component]; if (!schema) throw new Error(`Missing registered component: ${component}`);
  return { format: 'rustui.preview', format_version: 1, preview_id: id, component_id: component, component_version: schema.version, properties, actions };
}
function draft() {
  const component = $('component').value;
  if (capabilities.profile === 'preview') $('document').value = JSON.stringify(preview(component), null, 2);
  else $('document').value = JSON.stringify({ format: 'rustui.composition', format_version: 1, preview_id: 'browser-reference', root: 'root', nodes: { root: { preview: preview(component), slots: {} } } }, null, 2);
}
async function expectError(command, code) {
  try { await request(command); } catch (error) { if (error.code === code) { log(`PASS expected ${code}`); return; } throw error; }
  throw new Error(`Expected ${code}`);
}
async function roundtrip() {
  enabled(false);
  try {
    // These are test fixtures; the document editor and transport have no component ceiling.
    if (capabilities.profile === 'preview') {
      await request({ kind: 'mount', source: JSON.stringify(preview('text', { text: 'Browser reference mount' })) });
      const old = { ...stamp };
      await request({ kind: 'edit_properties', stamp, updates_source: '[["text","Browser reference edit"]]' });
      const exported = await request({ kind: 'export', stamp });
      if (JSON.parse(exported.source).properties.text !== 'Browser reference edit') throw new Error('Text edit did not roundtrip');
      await expectError({ kind: 'export', stamp: old }, 'stale_stamp');
      if (!capabilities.allowed_actions.includes('increment')) throw new Error('Owner did not allow increment fixture');
      await request({ kind: 'mount', source: JSON.stringify(preview('button', { text: 'Browser reference action' }, { activate: 'increment' })) });
      const result = await request({ kind: 'signal', stamp, event: 'activate', node: null });
      if (result.action !== 'increment') throw new Error('Wrong action routed');
    } else {
      const id = 'browser-reference';
      const node = (component, properties = {}) => ({ preview: preview(component, properties, {}, id), slots: {} });
      const source = JSON.stringify({ format: 'rustui.composition', format_version: 1, preview_id: id, root: 'root', nodes: { root: node('composition-column') } });
      await request({ kind: 'mount', source });
      const old = { ...stamp };
      const edits = [
        { kind: 'insert', parent: 'root', slot: 'body', node: 'first', value: node('composition-text', { label: 'First' }) },
        { kind: 'insert', parent: 'root', slot: 'body', node: 'second', value: node('composition-text', { label: 'Second' }) },
        { kind: 'set_property', node: 'first', property: 'label', value: 'Edited through browser' },
        { kind: 'move', node: 'first', parent: 'root', slot: 'body' },
        { kind: 'remove_subtree', node: 'second' },
      ];
      await request({ kind: 'edit_composition', stamp, edits_source: JSON.stringify(edits) });
      const result = await request({ kind: 'export', stamp }), doc = JSON.parse(result.source);
      if (doc.nodes.second || doc.nodes.first.preview.properties.label !== 'Edited through browser') throw new Error('Composition edits did not roundtrip');
      await expectError({ kind: 'export', stamp: old }, 'stale_stamp');
    }
    const previous = client.session;
    await client.disconnect(stamp); stamp = null;
    await connect();
    if (client.session === previous) throw new Error('Reconnect reused revoked session');
    status(`PASS canonical ${capabilities.profile} roundtrip, stale stamp, disconnect and reconnect. Inspect iframe render separately.`);
  } finally { enabled(!!client?.alive && !!client.session); }
}
function action(id, fn) { $(id).addEventListener('click', () => Promise.resolve().then(fn).catch(error => { log(`ERROR ${error.message}`); status(error.message); enabled(!!client?.alive && !!client.session); })); }
action('connect', connect); action('catalog', refresh); action('draft', draft); action('roundtrip', roundtrip);
action('mount', () => request({ kind: 'mount', source: $('document').value }));
action('edit', () => request({ kind: capabilities.profile === 'preview' ? 'edit_properties' : 'edit_composition', stamp, [capabilities.profile === 'preview' ? 'updates_source' : 'edits_source']: $('edits').value }));
action('export', async () => { $('document').value = (await request({ kind: 'export', stamp })).source; });
action('signal', () => request({ kind: 'signal', stamp, node: capabilities.profile === 'composition' ? $('node').value : null, event: $('event').value }));
action('disconnect', async () => { await client.disconnect(stamp); stamp = null; enabled(false); status('Disconnected and unmounted'); });
