# Owner-supplied RustUI macOS component reference

**Date recorded:** 2026-10-22  
**Provenance:** User message `Sentinel_2db9c054de248191bf47c9ee6ae389b6` in the RustUI HIG work conversation. The user introduced this as a reference for components RustUI should cover, “and maybe more.”  
**Purpose:** Required item-level coverage input for the hig-only macOS 27 theme design. This is a transcription of named concepts and claim families, not a verbatim copy of the entire original long message and not an authoritative Apple API specification. Examples and APIs are retained where useful; every named row must be reconciled, including system-owned and unsupported concepts.  
**Boundary:** All follow-up files, validation artifacts, and design changes must remain under `/Users/Abium/Developer/rustui/hig`. Do not modify production/source/runtime files or apply design output outside `hig/` without later explicit authorization.

## Reconciliation contract

For every source item, record at least:

- Source section and exact source label.
- Canonical concept ID and aliases or cross-references.
- Category and ownership: RustUI-rendered, native control, native integration, system-owned/non-reimplementable, composition, unsupported, or unresolved.
- Relevant API and macOS availability evidence; distinguish user claims, official Apple evidence, and secondary reporting.
- Current theme JSON status, current code/runtime status, and design status as separate fields.
- Proposed theme properties, semantic parts, variants, interaction states, sizes, light/dark and accessibility behavior, keyboard/input behavior, and localization/layout needs where applicable.
- Native fallback or explicit unsupported/unresolved decision.
- Validation or acceptance plan.

Deduplicate repeated ideas using aliases and cross-references rather than dropping source rows or counting aliases as new primitives. The existing 41 theme records are a preservation baseline, not a coverage ceiling. Add other concepts only with a stated purpose and evidence. Keep the design candidate distinct from the current runtime schema. Screenshots are illustrative design evidence, not Apple requirements or proof of RustUI support.

## Named-item inventory

The following list preserves all named items in the supplied reference. Parenthesized examples are compact API/variant hints from the source; they are not verified here.

### 1. System-level UI

- Menu bar (NSMenu, commands)
- Menu bar extra (NSStatusItem, MenuBarExtra)
- Dock (NSDockTile)
- Control Center (ControlWidget, WidgetKit)
- Notification Center (UNUserNotificationCenter)
- Widgets (WidgetKit; source claims App Intents and dynamic styling in macOS 27)
- Spotlight (Core Spotlight, App Intents)
- Apps app launcher (system; source claims it replaced Launchpad in macOS 26)
- Mission Control
- Stage Manager
- Window tiling
- Hot Corners
- Lock Screen / Login window
- Quick Look (QLPreviewPanel)
- Share sheet (NSSharingServicePicker, ShareLink)
- Services menu (source names NSServicesMenuRequestor; verify exact API)

### 2. Windows and presentation

- Window (NSWindow, WindowGroup, Window)
- Window controls / traffic lights (standardWindowButton)
- Title bar (NSTitlebarAccessoryViewController; source claims uniform toolbars in macOS 27)
- Panel (NSPanel, UtilityWindow)
- Sheet (beginSheet, .sheet)
- Alert (NSAlert, .alert)
- Confirmation dialog (.confirmationDialog)
- Popover (NSPopover, .popover)
- Open and Save panels (NSOpenPanel, NSSavePanel, .fileImporter, .fileExporter; suggested filenames reported for macOS 27)
- Settings window (Settings scene)
- Color panel / Font panel (NSColorPanel, NSFontPanel)
- About panel (orderFrontStandardAboutPanel)
- Window tabs (NSWindow.tabbingMode)
- Full-screen mode (toggleFullScreen)

### 3. Navigation and structure

- Sidebar (NSSplitViewItem sidebar, NavigationSplitView; edge-to-edge sidebar is a macOS 27 claim)
- Split view (NSSplitView, HSplitView, VSplitView)
- Inspector (NSSplitViewItem inspector, .inspector)
- Tab view (NSTabView, TabView)
- Tab bar (TabView; redesigned in macOS 27 is a source claim)
- Toolbar (NSToolbar, .toolbar)
- Toolbar item (NSToolbarItem, ToolbarItem)
- Toolbar item group (NSToolbarItemGroup, ToolbarItemGroup; semantic role in macOS 27 is a source claim)
- Search toolbar item (NSSearchToolbarItem, .searchable)
- Tracking separator (NSTrackingSeparatorToolbarItem)
- Toolbar spacer (flexibleSpace, ToolbarSpacer; macOS 26 source claim)
- Path control (NSPathControl)
- Page controller (NSPageController)
- Navigation stack (NavigationStack)

### 4. Menus

- Menu (NSMenu, Menu)
- Menu item (NSMenuItem; preferredImageVisibility in macOS 27 is a source claim)
- Submenu (NSMenuItem.submenu)
- Context menu (.menu, .contextMenu)
- Pop-up menu (NSPopUpButton, Picker.menu)
- Pull-down menu (NSPopUpButton pullsDown, Menu)
- Dock menu (applicationDockMenu)
- Status item menu (NSStatusItem.menu)
- Key equivalents (keyEquivalent, .keyboardShortcut)

### 5. Buttons and toggles

- Push button (NSButton, Button)
- Default / prominent button (keyEquivalent, prominence, .borderedProminent)
- Icon button (NSButton, SF Symbol)
- Pop-up button (NSPopUpButton, Picker)
- Checkbox (NSButton switch, Toggle.checkbox)
- Radio button (NSButton radio, Picker.radioGroup)
- Switch (NSSwitch, Toggle.switch)
- Disclosure triangle (NSButton disclosure, DisclosureGroup)
- Help button (NSButton helpButton)
- Segmented control (NSSegmentedControl, Picker.segmented)
- Color well (NSColorWell, ColorPicker)
- Share button (NSSharingServicePicker, ShareLink)
- Link

### 6. Text input and display

- Label (NSTextField labelWithString, Text, Label)
- Text field (NSTextField, TextField; styled prompts are a macOS 27 source claim)
- Secure text field (NSSecureTextField, SecureField)
- Search field (NSSearchField, .searchable)
- Text view (NSTextView, TextEditor; multiline and rich text)
- Combo box (NSComboBox)
- Token field (NSTokenField)

### 7. Pickers and selection

- Date picker (NSDatePicker, DatePicker; date/time, text, stepper, calendar, clock variants)
- Picker (menu, segmented, inline, radio variants)
- File importer / exporter
- Photos picker (PhotosPicker)
- Image well (editable NSImageView)

### 8. Values and progress

- Slider (NSSlider, Slider; linear, circular, tick marks)
- Stepper (NSStepper, Stepper)
- Level indicator (NSLevelIndicator; level, rating, capacity)
- Progress indicator (NSProgressIndicator, ProgressView; bar and spinner)
- Gauge

### 9. Lists and collections

- Table view (NSTableView, Table)
- Outline view (NSOutlineView, OutlineGroup, List children)
- List
- Collection view (NSCollectionView, LazyVGrid)
- Browser (NSBrowser column navigation)
- Scroll view (NSScrollView, ScrollView)
- Scroll bars (NSScroller)
- Pull-to-refresh (NSRefreshController; macOS 27 claim)
- Reorder by drag (reorderable(); macOS 27 claim)

### 10. Layout containers

- Stack view (NSStackView, HStack, VStack, ZStack)
- Grid view (NSGridView, Grid)
- Box / group box (NSBox, GroupBox)
- Separator (NSBox separator, Divider)
- Form
- Auto Layout (NSLayoutConstraint)
- Safe area / layout guide (NSLayoutGuide)
- Control sizes (mini, small, regular, large, extra large; macOS 26 source claim)
- Concentric corners (new macOS 27 API claim)

### 11. Feedback and status

- Tooltip / help tag (toolTip, .help)
- Badge (NSDockTile.badgeLabel, .badge)
- Notification banner (UNUserNotificationCenter)
- Cursor (NSCursor)
- Focus ring (system)
- Haptic feedback (NSHapticFeedbackManager)

### 12. Media and content views

- Image view (NSImageView, Image)
- SF Symbols (NSImage systemSymbolName, Image systemName)
- Video player (AVPlayerView, VideoPlayer)
- Web view (WKWebView, WebView)
- Map view (MKMapView, Map)
- PDF view (PDFView)
- Embedded Quick Look preview (QLPreviewView)
- Chart (Swift Charts, Chart)
- Canvas / Metal view (Canvas, MTKView)
- 3D view (RealityView)

### 13. Liquid Glass and materials

- Glass effect view (NSGlassEffectView, .glassEffect(); macOS 26 claim)
- Glass container (NSGlassEffectContainerView, GlassEffectContainer; macOS 26 claim)
- Glass button styles (.glass, .glassProminent; macOS 26 claim)
- Interactive glass (macOS 27 claim)
- Background extension (NSBackgroundExtensionView, .backgroundExtensionEffect(); macOS 26 claim)
- Scroll edge effect (macOS 26 / refined macOS 27 claim)
- Visual effect view (NSVisualEffectView)
- Control prominence and shapes (capsule, prominence; macOS 26 claim)
- Liquid Glass slider (clear-to-opaque; macOS 27 claim)
- Window corner radius (same radius across windows; macOS 27 claim)

### 14. Input and interaction patterns

- Drag and drop (NSDraggingSource, .draggable, .dropDestination)
- Gesture recognizers (NSGestureRecognizer; macOS 27 focus claim)
- Control events (NSControl.Events; macOS 27 claim)
- Text selection API for custom views (new macOS 27 claim)
- Keyboard navigation / full keyboard access
- Trackpad gestures (swipe, pinch, rotate, smart zoom)
- State restoration (NSWindowRestoration; macOS 27 guidance claim)
- Touch Bar (retired hardware)

### 15. System-provided popups

- Writing Tools
- Autofill (Passwords / OTP)
- Emoji and Symbols viewer
- Continuity Camera import / scan menu
- Look Up
- Siri (new macOS 27 app is a source claim)

### 16. App structure / SwiftUI scenes

- Window group (WindowGroup)
- Single window (Window)
- Document group (DocumentGroup; ReadableDocument/WritableDocument are macOS 27 source claims)
- Settings
- Menu bar extra (MenuBarExtra; cross-reference section 1)
- Utility window (cross-reference Panel in section 2)
- Commands (.commands; cross-reference Menu bar in section 1)

## Appearance requirements and design claims

The source requests coverage for:

- Light / Dark / Auto appearance.
- Icon and widget styles: Default, Dark, Clear, Tinted.
- Glass opacity control (macOS 27 claim: clear-to-opaque slider, medium default, no ultra-clear option reported).
- Accent/highlight colors, including Graphite, and wallpaper tinting.
- Reduce Transparency and Increase Contrast.
- Component-specific light/dark behavior for windows, sidebars, text, buttons, fields, shadows, and glass.
- Semantic system-resolved color roles, adaptive assets, and appearance updates.
- Keyboard/focus, disabled/inactive, selection, accessibility, localization, RTL, and native fallback behavior per relevant item.

Do not encode oversimplifications as universal rules: the source explicitly cautions against claims that every dark button lacks a border, shadows are always invisible, or custom bitmaps always require separate dark variants. Specify semantic roles and component-specific appearance behavior.

Potential mechanisms to examine, with availability verified before use: AppKit semantic NSColor roles; SwiftUI Color.primary and NSColor bridging; Any/Dark/HighContrast asset variants; NSApp.appearance/effectiveAppearance, viewDidChangeEffectiveAppearance/updateLayer; SwiftUI colorScheme/preferredColorScheme; template-symbol adaptation.

## Version-sensitive claim register

Verify each claim from official Apple documentation, SDK declarations, or WWDC sources before treating it as a fact or design requirement:

- macOS 27 codename “Golden Gate”.
- NSRefreshController.
- NSToolbarItemGroup.role.
- NSControl.Events.
- NSMenuItem.preferredImageVisibility.
- Interactive glass and sidebar, scroll-edge, and toolbar changes.
- Concentricity API.
- Custom-view text selection API.
- Keyboard/gesture changes and state-restoration guidance.
- SwiftUI reorderable(), ReadableDocument/WritableDocument, toolbar visibility priority/auto-minimizing, styled TextField prompts, and App Intents View Annotations.
- Press/community claims: Liquid Glass slider/default medium/no ultra-clear; edge-to-edge sidebars, colored icons, uniform toolbars, common window corner radius; darker edges/brighter highlights; icon glass layers for light/dark/tinted/clear; HDR/OLED behavior; launcher/battery icon changes; suggested filenames; iCloud toolbar icon; natural-language Shortcuts; new Siri app; NSScrollView touch-count properties.

For each, use a confidence state such as **confirmed**, **corrected**, **reported**, or **unverified**; include source and exact API/platform availability. Community reporting is only a lead. A local macOS 26.1 SDK cannot prove a macOS 27 declaration absent.

## User-supplied source links

Prefer current official Apple Developer guidance and SDK declarations. Treat the other sources as leads, not platform authority.

- https://developer.apple.com/macos/whats-new/
- https://developer.apple.com/videos/play/wwdc2026/289/
- https://developer.apple.com/videos/play/wwdc2025/310/
- https://mjtsai.com/blog/2026/06/18/appkit-in-macos-27/
- https://mjtsai.com/blog/2026/06/19/swiftui-in-appleos-27/
- https://forums.macrumors.com/threads/macos-27-all-the-little-things.2483520/
- https://www.macrumors.com/2026/06/09/macos-golden-gate-liquid-glass/
- https://9to5mac.com/2026/09/22/macos-27-gives-you-more-control-over-liquid-glass/
- https://eclecticlight.co/2026/05/31/last-week-on-my-mac-razzle-and-dazzle/

## Reconciliation checklist

- Preserve all 16 section headings and every named source item above in the resulting coverage matrix. Report total named rows using an explicit counting rule; do not claim completion solely from category counts.
- Deduplicate repeated concepts with aliases/cross-references: file importer/exporter, settings, menu bar extra, Quick Look, sharing, pop-up button/menu, and scenes appear more than once.
- Classify system desktop/security surfaces (Dock, Mission Control, Login UI, Spotlight, system popups, Control Center, system widgets) as integrations, system-owned, unsupported, or unresolved where appropriate; do not promise RustUI can recreate OS-owned surfaces.
- Compare against the preliminary gap catalogue and existing 41 v1 records. Add other justified concepts (for example validation messages, empty/loading/error states, command palette, virtualization, rich text/rulers, range/date/time variants, tree tables, pagination, breadcrumbs, splitter/resizers) without inflating alias counts.
- Keep schema candidates separate from the current runtime JSON format. Validate row references, aliases, IDs, source links, images, and provenance using static checks confined to hig.
- The source list is a coverage target, not verified fact. Distinguish requirement, Apple-documented behavior, community report, RustUI implementation evidence, design concept, and unresolved decision.
