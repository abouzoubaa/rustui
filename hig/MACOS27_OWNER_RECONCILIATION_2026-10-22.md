# Owner reference reconciliation — 2026-10-22

This is the design-only reading guide for [`MACOS27_OWNER_COMPONENT_COVERAGE_2026-10-22.csv`](MACOS27_OWNER_COMPONENT_COVERAGE_2026-10-22.csv). The matrix preserves the owner's complete named-item inventory and separates Apple evidence, current RustUI data/code, and candidate design status.

## Counting and coverage

- The count is **148 named-item rows**: one row per top-level bullet beneath the 16 numbered headings in `OWNER_COMPONENT_REFERENCE_2026-10-22.md`'s “Named-item inventory”. It preserves exact bullet text, including parenthesized API/variant hints. Repeated items stay in their source positions and link through canonical IDs, aliases, and cross-reference families.
- Section counts are 16, 14, 14, 9, 13, 7, 5, 5, 9, 9, 6, 10, 10, 8, 6, and 7. Their sum is 148.
- The separate “Appearance requirements”, “Version-sensitive claim register”, and “User-supplied source links” are captured as **28 supplemental rows** in [`MACOS27_OWNER_CLAIM_COVERAGE_2026-10-22.csv`](MACOS27_OWNER_CLAIM_COVERAGE_2026-10-22.csv): 8 + 11 + 9. These do not inflate the 148 named component rows or any primitive count. Reconciliation checklist bullets are validation instructions, not additional components or API claims.
- Matrix category counts use an explicit primary classification: 17 system-owned, 5 native-system integrations, 51 native-host/composition, 64 RustUI/style/composition, 10 native-material/API, and 1 retired/unsupported. A primary category does not override a row's explicit fallback or platform limitation.

## Evidence and runtime boundaries

`hig/macos27.json` remains byte-identical to `hig/macos27.production-baseline.2026-10-22.json` (SHA-256 `5a4a81adb6af226d59af916a4feadf575b23447b7cee2517882e9bdf2453e01e`), with 41 component IDs and 419 styles. The runtime registry declares 27 theme IDs mapped to renderer capabilities and 14 reference-only IDs. Mapping is selective: a mapped ID is a partial style contract, and the zero-style `toolbar`, `sidebar`, and `tab_bar` entries remain profile/GPUI-level rather than item-complete contracts. A related theme ID in the matrix is a crosswalk only; it is not proof of semantic equivalence or complete runtime behavior.

The v2 JSON remains `runtime_compatible: false`. Its existing 31 source-present contracts, 5 absent reusable patterns, 11 native integrations, and 9 composition examples retain their counts. The matrix links owner concepts into those existing candidates. Eight new cross-cutting design-contract groups were added as metadata, explicitly excluded from component counts. Nothing was copied to production themes or runtime code.

The candidate and gap catalogue also record eight owner-derived behavior-gap groupings: validation messages; empty/loading/error; virtualization; rich text/rulers; date/time/range variants; tree tables; pagination; and breadcrumb/splitter behavior. These are mapped to existing component candidates or explicitly deferred as patterns, not added as new primitive counts. The catalogue records their evidence and limits.

Subsequent design work is governed by [`MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md`](MACOS27_VISUAL_FIDELITY_ACCEPTANCE_2026-10-22.md). The objective is demonstrated fidelity to observed macOS 27 controls and official Apple guidance/APIs, not resemblance to generated images. The current catalogue and static checks do not pass that gate.

## Shared design contracts referenced by matrix families C01–C16

These contracts describe work to specify, not behavior already shipped.

| Family | Contract to specify |
|---|---|
| C01 System-level UI | Keep Dock, Control Center, Notification Center, Widgets, Spotlight, launcher, Mission Control, Stage Manager, tiling, Hot Corners, and login surfaces system-owned. App contributions use the corresponding supported integration or extension; do not imitate OS chrome. Menu commands, menu extras, Quick Look, sharing, and Services are host/native handoffs. |
| C02 Windows and presentation | Separate app content, titlebar, toolbar, window controls, document tabs, panels, sheets, popovers, settings, and native file/color/font/about panels. Define active/inactive, resized, full-screen, modal, cancellation, focus entry/return, and default/cancel/destructive actions. Keep system-owned controls with the host. |
| C03 Navigation and structure | Give sidebar, split, inspector, tabs, toolbar, toolbar items/groups, search, tracking separators/spacers, path controls, page controllers, and navigation stacks distinct semantic roles. Specify selection, collapse, overflow, resizing, keyboard route, focus retention, RTL order, and native toolbar customization separately. |
| C04 Menus | Specify command hierarchy, checked/mixed/disabled items, submenus, contextual and pull-down variants, key equivalents, dismissal, focus, and shortcut display. Do not conflate Dock/status-item menus or the menu bar host with an ordinary popup. |
| C05 Actions and choices | Specify action role/prominence, icon-only accessible names, default/destructive states, popup/checkbox/radio/switch/disclosure/help/segmented/color/share/link semantics, focus and disabled states, group behavior, and keyboard activation. Segmented selection is not automatically tab navigation. |
| C06 Text entry and editing | Specify visible/programmatic labels separately from prompts; caret, selection, composition/IME, validation/error/help, secure reveal, search clearing/suggestions, multiline/rich editing, combo/token entry, bidi, scaling, and accessible announcements. |
| C07 Selection and native pickers | Specify locale/calendar/time-zone/date-range and clock/date variants, picker selection/keyboard rules, sandboxed file import/export cancellation, photo authorization, image-well edit/drop states, and native-panel handoff. |
| C08 Range and progress values | Specify min/max/step/value, linear/circular orientation, ticks, keyboard increments, bounds, rating/capacity semantics, determinate/indeterminate/paused/failed progress, and value announcements. Gauge is a data/semantic display, not automatically a slider. |
| C09 Collections and scrolling | Distinguish table, tree/outline, list, grid, browser columns, scroll container, and scrollbar. Specify row/column focus, sorting, selection, reorder, virtualization, stable identity, pull-to-refresh lifecycle, empty/loading/error, and accessible alternatives. |
| C10 Layout and sizing | Specify intrinsic/responsive stack/grid/form/group/separator behavior, min/max constraints, safe-area/layout-guide ownership, density and control-size policy, overflow, wrapping, and corner-concentricity as a native API concept. Avoid presenting Auto Layout as a theme widget. |
| C11 Feedback and status | Separate tooltip/help, badge, in-app notification and delivered system notification. Specify cursor/focus ownership, severity, timing, action, announcement throttling, reduced motion, and haptic availability/fallback. |
| C12 Media and content | Specify informative/decorative images and symbol modes, video playback/captions/keyboard, web/map/PDF/Quick Look native ownership, chart non-color encodings/data alternative, canvas/Metal interaction and 3D-view host requirements. Candidate screenshots are not implementation evidence. |
| C13 Materials and appearance | Treat Liquid Glass, interactive glass, background extension, scroll-edge effects, and native visual-effect views as platform APIs. Existing profile opacity/radii do not implement them. Specify adaptive semantic colors, component-specific light/dark, system appearance, accent, contrast/Show Borders, Reduce Transparency, and fallback surfaces. |
| C14 Input and restoration | Specify drag source/destination, gesture intent, control event delivery, text selection, keyboard/full-keyboard-access and precision trackpad alternatives, focus continuity, persistence, and cancellation. Touch Bar is retired hardware and excluded from new macOS design. |
| C15 System-provided assistance | Writing Tools, password/OTP autofill, Emoji/Symbols, Continuity Camera, Look Up, and Siri are system-provided surfaces. Integrate with host capabilities and permissions; do not recreate their interface as RustUI controls. |
| C16 Scenes and cross-cutting requirements | Treat WindowGroup/Window/DocumentGroup/Settings/MenuBarExtra/UtilityWindow/Commands as app-scene or host structure, not ordinary theme components. Appearance, accessibility, localization, input, and fallback rules apply across the relevant rows. |

### Cross-cutting appearance, accessibility, and locale requirements

The owner's appearance bullets are retained as design requirements: light/dark/system-resolved appearance; icon/widget style choices only where the host exposes them; semantic accent/highlight and wallpaper-tint inputs; Reduce Transparency and Increase Contrast/Show Borders; per-component adaptive colors/material behavior; and updates when environment appearance changes. `Auto` must resolve from the host environment rather than be a hard-coded color mode. Graphite and wallpaper tint are inputs to investigate, not promised palette tokens. A glass clear-to-opaque control/default is unresolved and must not be encoded as a macOS rule.

Every applicable row must specify accessible name, role, value, keyboard operation, focus visibility and return, disabled/inactive/selected/expanded state, contrast, and screen-reader behavior. Text and collections additionally need IME, bidi, locale formatting, announcements, stable identity, and larger-text behavior. Layout must support localization expansion and RTL direction. Theme data cannot itself prove these interaction/accessibility properties.

## Apple claim dispositions

These are source-checked against Apple primary sources on 2026-10-22. A source label or API hint remains user-supplied until individually verified. The 11 version-claim rows and eight appearance rows are preserved with individual status in the supplemental matrix. “Confirmed” confirms only the scoped platform fact, not RustUI support.

| Claim | Disposition | Evidence and scope |
|---|---|---|
| macOS 27 “Golden Gate” codename | Confirmed as Apple release-note title. | [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes). |
| `NSRefreshController` pull-to-refresh for `NSScrollView` | Confirmed, macOS 27.0 AppKit. | [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes), [NSRefreshController](https://developer.apple.com/documentation/appkit/nsrefreshcontroller). This is native AppKit behavior; the RustUI theme record is not an implementation. |
| `NSToolbarItemGroup.role` and segmented-control `role` including tabs | Confirmed in macOS 27 release notes. | [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes). Preserve toolbar grouping, segmented selection, and tab navigation as distinct semantics. |
| `NSMenuItem.preferredImageVisibility` | Confirmed; SDK-linked behavior is conditional and AppKit can still hide an image. | [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes), [property reference](https://developer.apple.com/documentation/appkit/nsmenuitem/preferredimagevisibility). Do not impose a universal menu-image rule. |
| `NSControl.Events` | API and event semantics confirmed; introduction in macOS 27 not established. | [NSControl.Events](https://developer.apple.com/documentation/appkit/nscontrol/events). |
| `NSTextSelectionManager` for custom text views | Confirmed as a new macOS 27 AppKit API. | [Modernize your AppKit app, WWDC26](https://developer.apple.com/videos/play/wwdc2026/289/), [NSTextSelectionManager](https://developer.apple.com/documentation/appkit/nstextselectionmanager). |
| `ReadableDocument` / `WritableDocument` and styled SwiftUI `TextField` prompts | Confirmed in macOS 27 SDK release notes. | [macOS 27 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-27-release-notes). |
| Interactive glass, sidebar/toolbar/scroll-edge refinements, and corner-concentricity | Confirmed as macOS 27 AppKit guidance. | [Modernize your AppKit app, WWDC26](https://developer.apple.com/videos/play/wwdc2026/289/). The session describes native APIs and selective interactive use, not a RustUI style token. |
| ControlWidget / WidgetKit controls on Mac | Confirmed system-space integration concept; macOS 27 introduction is not claimed. | [WidgetKit Controls](https://developer.apple.com/documentation/widgetkit/controls-collection). WidgetKit, SwiftUI templates, and App Intents supply system controls; not an in-app RustUI widget implementation. |
| View Annotations | Apple’s WWDC26 macOS guide describes a View Annotations capability for mapping views to entities. The exact public API symbol and availability need SDK-reference confirmation. | [WWDC26 macOS guide](https://developer.apple.com/wwdc26/guides/macos/). This is distinct from the App Intents Testing `ViewAnnotation` type previously discussed in the claim register; neither establishes a general RustUI annotation API. |
| `NSServicesMenuRequestor` spelling; replacement launcher; styled prompts, suggested filenames, special Liquid Glass slider/defaults, exact common radius, HDR/OLED and icon-layer reports; `NSScrollView` touch-count properties; new Siri app; and other press/community items | Unverified here. | Preserve as leads only. No design or support claim until an exact Apple API/release note/WWDC source and platform availability support it. |
| `NSWindowRestoration` and keyboard/gesture guidance | Existing native concepts; WWDC26 guidance is confirmed, but not a new macOS 27 API claim. | [Modernize your AppKit app, WWDC26](https://developer.apple.com/videos/play/wwdc2026/289/). |
| SwiftUI `.reorderable()`, toolbar minimization/visibility priority, suggested Liquid Glass slider defaults | Unverified in this reconciliation. | Keep as research leads; require exact Apple declaration/release note before adding an API or default to the candidate. |
| View Annotations | WWDC26 describes the capability; exact symbol/availability remains unresolved. | [WWDC26 macOS guide](https://developer.apple.com/wwdc26/guides/macos/); distinguish the capability from App Intents Testing's `ViewAnnotation` test type. |

The previous API claim register's statement that the 16-section inventory was not present is superseded by this reconciliation. Its API-by-API caution remains in effect.

## Static verification

[`validate_owner_component_coverage.py`](validate_owner_component_coverage.py) verifies that all 148 exact source labels and section positions are represented once, required classification fields are populated, the candidate points to the matrix and remains runtime-incompatible, and `hig/macos27.json` still matches the preserved production snapshot at 41 IDs / 419 styles. It is a file/JSON consistency check only; no build, test, GUI, accessibility, or macOS 27 runtime acceptance is claimed.
