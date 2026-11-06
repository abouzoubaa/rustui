# M1322 — Stage 9 registry acceptance run sheet

## Acceptance target

Verify the existing Unicode `String` default contract for the preview
registry: a default longer than `max_length` is rejected without reserving the
component ID, and a corrected default under the same ID can register, mount,
and render. This run sheet adds no milestone and does not change the D1b or D1d
gates.

## Prerequisites

- Use the M1322 claim worktree and the exact source under review.
- Use `./scripts/cargo-headstart` with `--locked --offline`.
- The previous test run used a temporary vendor overlay because four files
  were absent from the local `cc-1.8.0` package. Before repeating the checks,
  recreate a temporary overlay without modifying `vendor/`; verify every
  manifest-listed replacement file against the local checksum manifest, then
  remove only the overlay symlinks after testing.
- The test schema must declare a String property with `max_length: 2` and
  default `ééé`. Its corrected same-ID schema uses default `éé`.

## Source-level outcomes

Record each expected outcome separately; the existing focused regression
passed all three:

1. Registration rejects `ééé` because it contains three Unicode scalar values
   while `max_length` is two.
2. Rejection leaves the registry catalog empty, and the corrected schema
   registers under the same component ID.
3. Mounting resolves the exact corrected default `éé`, and the host render
   path returns a view.

The corrected value is two Unicode scalar values encoded as four UTF-8 bytes.
This distinguishes character-count behavior from byte-count behavior. The
test does not establish extended grapheme-cluster semantics.

Focused command:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests::overlong_unicode_string_default_keeps_registry_available_for_corrected_schema --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1322"' -- --exact --quiet
```

Module command:

```sh
./scripts/cargo-headstart test -p rustui --lib preview::registry::tests --locked --offline --config 'source.vendored-sources.directory="/tmp/rustui-stage9-vendor-1322"' -- --quiet
```

Previously recorded results were 1/1 focused and 10/10 for the registry test
module. Full output is in:

- `target/headstart/evidence/M1322-preview-registry-unicode-max-length-default.log`
- `target/headstart/evidence/M1322-preview-registry-tests.log`

## Runtime and application evidence

The source-level test executes `PreviewRegistry` and `PreviewHost` in-process,
including mount, snapshot resolution, and render. It does not exercise the
Playground UI, a native application bundle, or a browser host.

Native or browser evidence is not required to establish this registry
invariant. If a later claim covers Playground or browser integration, first
provide a reachable fixture using this schema, then record the application or
browser version, source revision, built artifact identity, route or story,
observed registration/recovery result, and the corresponding run log. No such
GUI or browser run is performed by this sheet. D1b IME ownership and D1d
composition/external-host decisions remain gated.

## Evidence fields and limits

For any rerun, record the date, branch and source revision, OS and Rust toolchain,
exact command and exit status, `max_length`, Unicode scalar counts and UTF-8
byte counts, rejection result, catalog state after rejection, corrected
same-ID registration result, mounted property value, render result, and log
path. Keep source-level evidence distinct from application-level evidence.

This check covers the bounded registry/default/mount path. It does not establish
arbitrary component-catalog coverage, descriptor-import behavior, native
visual fidelity, accessibility, or cross-platform behavior.
