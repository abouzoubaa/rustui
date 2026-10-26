# D1d: RustUI Playground Browser Preview Host v1

The implementation-defined reference consumer is **RustUI Playground Browser
Preview Host**. Its transport is bounded, same-origin browser `postMessage` to
an iframe running the canonical Playground Web/Wasm application. This selects
a concrete reference consumer within the approved Web scope; it does not claim
the user selected a business application, native IPC, or arbitrary state export.

The adapter lives in `apps/rustui-playground`. It uses public RustUI preview
hosts, registered factories, descriptor codecs and GPUI owner scheduling.
RustUI has no dependency on this consumer, Chorus, or UI Studio.

## Contract and implementation evidence

All **10/10 reference-host implementation contracts** below are coded. Evidence
is deliberately separated by type; this count is not a project percentage or
complete browser/platform acceptance.

| Contract | Current evidence |
|---|---|
| Named host, transport and version | This specification; `rustui.preview.postmessage`, version 1; exact consumer ID `rustui.playground.browser-preview-host`. |
| Capability negotiation and registered catalog | Real iframe advertises commands, formats, limits, actions and 81 compiled component schemas; composition profile advertises its actual slot contracts. |
| Origin, source and handshake identity | Both JS ends pin exact origin and WindowProxy; opaque/top-level receivers are rejected. Cryptographic nonce/session handshake and source/origin rejection have executable JS fixtures. |
| Bounded strict messages and lossless source | One inflight request, 1 MiB UTF-8 envelope limit, strict Rust decoding, decimal-string u64 stamps/sequences; descriptor/edit/catalog sources remain JSON strings. Rust/JS rejection and precision fixtures pass. |
| Compiled mount, property edits and export | Real-browser canonical Text roundtrip passes; generic Badge mount exposes Inbox in the iframe AX tree. Factories are reused from the existing gallery. |
| Structural composition | Existing CompositionHost validates insert, set-property, move and remove-subtree; real-browser canonical edit/export roundtrip passes. Separately supplied child text was observed in AX and rendered pixels. |
| Explicit owner action policy | Preview owner advertises only increment; real-browser signal increments owner activations to 1. Registered dismiss action is denied without incrementing it. Composition has an empty action allowlist. |
| Stale revisions and request ordering | Real-browser old-stamp export is rejected; Rust tests cover replay, session rotation and stale sessions, with JS request/session/sequence identity tests. |
| Disconnect and owner lifecycle | Real disconnect unmounts the preview and disables host commands. Reconnect replaces iframe/session. JS fixtures cover pagehide, listener removal, reload and pending rejection; Rust bridge holds only weak owner/callback references. |
| Protected owner state | Actual gallery test and real browser reject remote SecureField value configuration; exported descriptor contains placeholder only. No InputState introspection or general owner snapshot is exposed. |

The public `OwnerAction::dispatch_with` hook returns a typed reply through the
existing weak foreground updater, including valid replies with no notification.
It returns None for a released/unavailable owner. Its real-owner tests are
recorded separately from the browser protocol fixtures.

## Run and validate

Build with the approved Headstart/Wasm environment using
`./scripts/verify-release.sh web`, serve `target/headstart/playground-web`, and
open `/reference-host.html`. Connect either profile, then use **Run canonical
roundtrip**. The generic document editor accepts schemas from the connected
catalog; canonical Text/Button fixtures are not its implementation ceiling.
Inspect the iframe render separately because transport success alone is not
rendering acceptance. Reconnect after a disconnect or iframe reload.

`ReferenceClient.disconnect()` uses the last stamp accepted from a successful
reply, including mount, property/composition edits, export and capabilities.
Replies without a stamp and rejected requests preserve that value; disposal
clears it. Passing an explicit stale stamp still rejects without disposing the
client. If local iframe interaction advances the revision independently, request
capabilities to obtain its current stamp before retrying a stale disconnect.

Focused checks run on 2026-10-26:

- Rust protocol tests: 5/5; actual gallery privacy/catalog: 1/1; composition
  metadata: 1/1; JS transport/client fixtures: 8/8, including default teardown
  after stamped replies in both profiles and preservation after stale errors.
- Conditional component default regression: 1/1; macro unit tests: 40/40.
- Native Playground all-target check, optimized Web build and Web packaging
  shell fixture passed. Cargo used locked/offline approved sources.
- Browser observations came from the in-app Chromium browser and current
  packaged iframe. Evidence/logs: `/tmp/rustui-preservation-20261025/browser/d1d-reference-v1`
  and `/tmp/rustui-d1d-*.log`.

Full supported-browser, screen-reader, real OS IME/clipboard, locale/RTL/font
and performance acceptance remain open. Native IPC and arbitrary Rust
view/closure/private-state serialization are outside this reference contract.
The release still has dependency-notice, signing and reproducibility gates.
