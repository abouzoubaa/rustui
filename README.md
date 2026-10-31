# RustUI

RustUI is a standalone, SwiftUI-inspired Rust GUI framework built on GPUI and
GPUI Components. It is MIT licensed and under active development.

## Current status


Core source includes retained components, inputs, collections, navigation,
overlays, graph/node editing, ICU4X/Fluent localization and descriptor-first
compiled composition editing. The canonical Playground exercises the public API.
External preview-host integration and release acceptance remain incomplete.
Behavioral, visual, accessibility, performance, and platform acceptance also
remain open. Stage 8 includes retained, virtualized `List`,
`Table`, and `Tree` controls with controlled selection.

For an independent application consuming this checkout, see
[Consuming RustUI from source](docs/CONSUMING_FROM_SOURCE.md). The standalone
example owns its required dependency patches and uses only the public RustUI API.

`ComponentHost` retains keyed GPUI entities and subscriptions. `TextField`,
`SecureField`, `SearchField`, and `TextEditor` preserve editor state across
body rebuilds. `Toggle`, `Checkbox`, and `Radio` use typed `Binding<bool>` values.
`Slider` retains upstream slider state and synchronizes a `Binding<f32>`; `Stepper`
updates a bounded `Binding<i32>` through GPUI Components buttons. `Picker` and
`ComboBox` retain upstream selection entities and write selected strings through
`Binding<String>`. `DatePicker` binds `Option<NaiveDate>` and `ColorPicker` binds
`Option<Hsla>` while retaining their upstream popup state.
`List` retains GPUI Components virtualization, search, focus, scrolling, and keyboard
selection state. `Table` retains upstream virtual rows, column sizing, scrolling,
focus, keyboard navigation, and row selection state.
`Tree` retains upstream hierarchy expansion, virtual scrolling, focus, keyboard
navigation, and stable-ID selection state.
`TabView` composes the upstream tab bar with controlled stable-ID selection and a
selected RustUI child.
`Breadcrumb` maps ordered RustUI component actions onto the upstream accessible
breadcrumb presentation.
`NavigationStack` uses a bound route path, heterogeneous destinations, and GPUI
Components buttons for forward and back navigation.

Localization preserves legacy interpolation and adds explicit Project Fluent resources,
ICU4X decimal and civil date/time formatting, and Unicode locale preferences.
See [the locale runtime contract](docs/tasks/STAGE6_LOCALE_RUNTIME_IMPLEMENTATION.md)
for supported functions, data versions and current acceptance limits. Native and
browser accessibility, broader platform acceptance and release gates remain open. RustUI has no direct,
transitive, optional, or public API dependency on Allui. GPUI and GPUI Components
are the upstream implementation sources.

## Run

```sh
./scripts/cargo-headstart playground
./scripts/cargo-headstart dev
./scripts/cargo-headstart lab
```

RustUI Playground is the canonical component development and manual acceptance
application. It currently exposes 15 live framework demos and 46 catalog rows, including
Outline, Drawer, GraphView, NodeEditor, and a directly searchable
SplitView/ResizablePanel gallery. The original theme-reference catalog remains
available alongside the additional public-API entries. Story source, events, English/French/Arabic locales, and Light/Dark
appearance controls are available. Layout and performance inspection remain
incomplete, and GUI acceptance is pending.

The `rustui-lab` package and `lab` alias are compatibility launchers for the same
Playground UI. They do not contain a second story implementation. See the
[Playground guide](docs/RUSTUI_PLAYGROUND.md) and [Lab compatibility note](docs/RUSTUI_LAB.md).

## Build

```sh
./scripts/cargo-headstart build --workspace --locked
./scripts/cargo-headstart check --workspace --all-targets --locked
python3 scripts/verify_dependency_reuse.py target/headstart/evidence/reuse-confirmed.jsonl
```

Headstart is experimental and uses the approved metadata-hash workaround.
Dependency sources stay in `vendor/`; build artifacts and reuse evidence stay in
`target/headstart/`.

Approved upstream stack:

- `gpui-pre = 0.3.9`
- `gpui-pre-platform = 0.3.9`
- `gpui-component = 0.7.3`

## Workspace packages

| Package | Responsibility | Status |
| --- | --- | --- |
| `rustui` | Public declarative API, component adapters, layout, theme and GPUI integration | Partial |
| `rustui-core` | Action, identity, state, binding, invalidation and environment contracts | Initial implementation |
| `rustui-macros` | `#[component]`, `VStack!`, and `HStack!` | Initial implementation |
| `rustui-i18n` | Legacy/Fluent catalogs, ICU4X number/date/time formatting, locale preferences and validation | Runtime implemented; locale/RTL/font acceptance pending |
| `rustui-rgpe` | Performance diagnostics and approved optimization integration | Scaffold |
| `rustui-playground` | Canonical component development app and macOS 27 reference catalog | GUI acceptance pending |
| `rustui-lab` | Compatibility launcher delegating to RustUI Playground | Compatibility |

## Project map

- [Architecture](docs/ARCHITECTURE.md)
- [Public API](docs/PUBLIC_API.md)
- [Authoritative file and capability registry](docs/FILE_REGISTRY.md)
- [Component catalog](docs/COMPONENT_CATALOG.md)
- [Implementation roadmap](docs/IMPLEMENTATION_ROADMAP.md)
- [Upstream audit](docs/UPSTREAM_AUDIT.md)
- [RustUI Playground](docs/RUSTUI_PLAYGROUND.md)
- [RustUI Lab compatibility](docs/RUSTUI_LAB.md)
- [Build and dependency reuse](docs/BUILDING.md)
- [Release verification and packaging](docs/RELEASE.md)

Implementation proceeds by coherent capabilities. An authorized capability may
change all related files needed to complete and validate it; unrelated work and
dependency revisions remain outside that scope.

Accessibility and VoiceOver acceptance are explicitly undone and deferred by
user direction. Functional and release work continues; see the
[canonical remaining-work record](docs/tasks/ROADMAP_REMAINING_AUDIT.md).
