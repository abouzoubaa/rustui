# Sidebar accessible-name validation

`SidebarItem::new` rejects empty or whitespace-only visible labels, and
`SidebarItem::localized` rejects empty or whitespace-only message IDs. A
localized item retains its original nonblank visible label; if the catalog
resolves its message to blank text, rendering uses that fallback instead of
panicking or creating an unnamed row. `SidebarView::accessibility_label`
rejects blank tree names while preserving nonblank caller text.

Focused source tests cover blank visible labels, blank localization IDs,
fallback from a blank resolved message, localized text precedence, blank tree
names, and preservation of nonblank tree names. They do not establish native
accessibility-tree or screen-reader acceptance.

Verification: `./scripts/cargo-headstart test -p rustui --lib sidebar::tests`
passed 9 tests. Rustfmt and the scoped diff check passed.

## Native keyboard and accessibility probe (M417)

The canonical Playground probe moved selection from Inbox to Projects with
Down, skipped disabled Settings, and returned to Inbox with Home and Up. End
and Enter left Projects selected. The accessibility tree exposed Inbox and
Projects as selectable, while Settings had no selectable action. Tab moved the
visible focus ring to Dark; a following Down did not change Sidebar selection.

This is partial native evidence only. AX captures reported the window root as
focused, so Sidebar focus identity was not verified. VoiceOver output was not
verified. Evidence: `target/headstart/evidence/sidebar-keyboard-probe-m417-20261021.json`.


### Focus-path source review (M417)

`SidebarView` keeps a retained root focus handle, marks it as a tab stop, and
tracks it on the Tree-role root. Its navigation handler runs only while that
handle is focused. Therefore Tab moving focus to Dark and a following Down key
leaving selection unchanged is expected. The root has an element identity and
Tree role, but the public Sidebar contract does not promise a platform
accessibility identifier. Native focus identity and VoiceOver announcements
remain unverified; no source change was justified by this review.

## M471 — Localized default navigation tree name

`Sidebar` and `NavigationSplitView` now name their navigation tree with the
`rustui-navigation-tree` message. English and French use `Navigation`; Arabic
uses `التنقل`. Missing or blank message results use the literal `Navigation`
fallback. Literal `.accessibility_label(...)` overrides remain supported, and
both views accept caller-selected localized message keys.

Source tests cover the default key and fallback, plus literal and localized
override preservation. The catalog contract resolves the framework name in
en, fr, and ar. These checks do not prove that the name appears in the native
accessibility tree or is announced by VoiceOver. Native accessibility,
screen-reader, and GUI acceptance remain open.

## N2 acceptance preparation — 2026-10-24

The canonical Sidebar gallery now puts disabled Settings between enabled Inbox
and Projects, includes a catalog-backed Components row using
`lab-navigation-components`, and includes a missing-key row whose visible
fallback is `Fallback sample`. The catalog-backed row resolves to Components in
English, Composants in French, and المكونات in Arabic. This gives the native
acceptance path an actual localized item and an unresolved-key fallback; the
other existing gallery labels remain literal.

Source evidence: `./scripts/cargo-headstart test -p rustui --lib
sidebar::tests --locked --offline` passed 17/17. The focused Playground check
`./scripts/cargo-headstart check -p rustui-playground --bin rustui-playground
--locked --offline` passed; it reported four dead-code warnings in
catalog/story support code. These checks do not verify rendered labels,
keyboard focus, the native accessibility tree, or VoiceOver.

An additional isolated source regression passed 1/1 in managed worktree
`/Users/Abium/.codex/worktrees/n2-sidebar-regression/rustui`:
`./scripts/cargo-headstart test -p rustui-playground --lib
sidebar_localized_labels_resolve_in_each_playground_locale`. It resolves
`lab-navigation-components` to Components, Composants, and المكونات, and
leaves `lab-sidebar-missing` unresolved for English, French, and Arabic. The
test source remains only in that worktree; it was not copied into the shared
checkout. This checks catalog resolution, not rendered labels or native
acceptance.

The source regression `normal_fallback_label_row_remains_selectable_by_stable_id`
passed 1/1 when the coordinator ran this filtered command in a dedicated
persistent target:

```sh
./scripts/cargo-headstart test -p rustui --lib normal_fallback_label_row_remains_selectable_by_stable_id --locked --offline
```

It uses the `missing-label-fallback` row, checks that End selects its stable ID,
Down keeps it selected at the end, and Enter preserves the current selection.
This is source-level selection coverage only; native rendering, keyboard event
delivery, the accessibility tree, and VoiceOver acceptance remain open.

Native acceptance remains pending. The M1223 Stage 5 acceptance session ended
blocked and did not release the shared app slot. Its recorded dependency was
Stage 6 cleanup plus the Stage 7 process watcher reporting the Playground
processes gone. The last recorded process snapshot listed PIDs 10507, 23568,
and 36564 (primary checkout, `c611` worktree, and `66c9` validation bundle).
That snapshot is historical, not a current slot signal. Do not launch, attach,
or replace an app until Stage 5 explicitly reports the slot free and a fresh
process check confirms the protected Playground processes have ended. M417's
gallery/build evidence predates this fixture change and is not native evidence
for the current story.

Current status refresh (2026-10-24): PID 10507 from the primary checkout,
PID 23568 from worktree `c611`, and PID 36564 from worktree `66c9` are still
running. Process metadata does not identify the originating chats. Treat the
slot as occupied; no GUI action was run. The established `M56 Review.app`
target and N2 native evidence record are absent.

## Stage 8 N2 native run sheet and evidence oracle

### Preflight and app identity

The current fixture was refreshed on 2026-10-24 without touching the shared app
executable:

```sh
CARGO_TARGET_DIR="$PWD/target/headstart/n2-prebuild" \
  ./scripts/cargo-headstart build -p rustui-playground \
  --bin rustui-playground --locked --offline
```

It exited successfully with four existing dead-code warnings. The resulting
`target/headstart/n2-prebuild/debug/rustui-playground` SHA-256 is
`7db471997a1f86eee9f19315da2122c7857b1a5bcb3dd8c57fb209734792f570`.
The standalone executable under the normal `target/headstart/debug/` remains
unchanged and predates this fixture.

Use the established disposable validation target
`target/headstart/verification/M56 Review.app` with bundle ID
`dev.rustui.playground.validation`. The regular `package-app.sh` output uses a
different path and the canonical `dev.rustui.playground` ID, so do not launch
that app for this probe. After Stage 5 release and the fresh process check,
package the prebuilt executable to one unlaunched staging bundle under
`n2-prebuild`, change its ID to `dev.rustui.playground.validation`, then move
that same bundle to the established validation target. Launch only the final
M56 Review path; do not create or launch another validation app. For example:

```sh
CARGO_TARGET_DIR="$PWD/target/headstart/n2-prebuild" \
  apps/rustui-playground/package-app.sh
staged="$PWD/target/headstart/n2-prebuild/debug/RustUI Playground.app"
/usr/libexec/PlistBuddy -c \
  'Set :CFBundleIdentifier dev.rustui.playground.validation' \
  "$staged/Contents/Info.plist"
plutil -lint "$staged/Contents/Info.plist"
mkdir -p "$PWD/target/headstart/verification"
validation="$PWD/target/headstart/verification/M56 Review.app"
test ! -e "$validation" || {
  printf '%s\n' "Remove only the released disposable validation bundle before moving." >&2
  exit 1
}
mv "$staged" "$validation"
```

Verify the final bundle ID before launch. Never launch the staging path or
overwrite a bundle/process until the slot-release and fresh-process conditions
are met. If the validation destination still exists, verify it is the known
disposable bundle and remove it only after confirming no protected process uses
it; do not allow `mv` to nest the staging bundle inside it.

### UI and keyboard sequence

1. Launch only the validation app above. In the Playground catalog, select
   **Sidebar** (catalog key `sidebar`). Leave the story defaults intact:
   Inbox, disabled Settings, Projects, localized Components, and Fallback
   sample; Projects is present and selection starts empty.
2. Click the **Inbox** row to select it and focus the Sidebar. Confirm the
   readout is `Selected ID: inbox`. Click disabled **Settings**; the readout
   must remain `Selected ID: inbox`. Press Tab until an outside control's
   focus ring is visible (M417 previously observed **Dark**), then press Down;
   the readout must still be `Selected ID: inbox`. Click **Inbox** again, press
   Down, and confirm the readout becomes `Selected ID: projects`. This proves
   Down skips disabled Settings and that the Sidebar handler responds only
   while its focus handle is focused.
3. With the Sidebar focused and Projects selected, press Enter and confirm
   Projects remains selected; press Down and confirm
   `Selected ID: localized-components`. Press Home and confirm `inbox`; press
   Up and confirm it remains `inbox`; press End and confirm
   `missing-label-fallback`; press Down and confirm it remains the last row.
   Record each readout; do not infer selection only from highlight color.
4. For localization, select the **Alert** catalog item (key `alert_dialog`),
   which contains the Playground locale buttons. Choose **English**, return to
   **Sidebar**, and capture the Sidebar. Repeat with **Français** and
   **العربية**, returning to **Sidebar** after each change. The locale action
   is global; verify the rendered Sidebar after every return.

### Pass/fail oracle and evidence

The N2 visual/keyboard portion checks that the visible Sidebar exposes rows in
this order: Inbox, Settings, Projects, localized Components, Fallback sample.
Components must read **Components** (English),
**Composants** (French), and **المكونات** (Arabic). Fallback sample remains
literal in every locale because `lab-sidebar-missing` is intentionally absent.
Settings must be unavailable for selection and must not change the selected-ID
readout when clicked. Activating Components or Fallback sample must set the
readout to its semantic ID: `localized-components` or
`missing-label-fallback`, respectively. Missing rows, wrong labels, or other
unexpected selection changes fail the N2 UI checks.

Stage 7 owns the native accessibility-tree capture and assertions. That step
was unavailable to this task-note update: no browser/app was inspected and no
N2 AX snapshot was obtained here. The required Stage 7 evidence must establish
the native tree/outline role and name (**Navigation** in English and French;
**التنقل** in Arabic), row roles and order, localized Components label, the
Fallback sample label, and Settings' disabled/selectable state. Record selected
state only if the native snapshot exposes it. Until Stage 7's current-fixture
capture is attached, all of those native tree claims remain unverified. The
older M417 AX snapshot reported the window root as focused and predates this
fixture; it is not a substitute. Neither this note nor M417 claims VoiceOver
speech acceptance.

The N2 evidence record at
`target/headstart/evidence/sidebar-n2-native-acceptance-20261024.json` does not
exist yet. Create it only after the N2 UI checks have actually run; include the
exact build/source hashes, validation bundle path and ID, locale results,
keyboard readouts, and cleanup result. Reference Stage 7's native tree evidence
by its actual path and SHA-256; do not reconstruct or claim a tree snapshot if
Stage 7 has not supplied one. Record the new evidence hash only after the file
is written.

After the GUI run, quit the app, remove the disposable validation bundle and
the standalone N2 executable, and verify no test app remains running. Preserve
the evidence and Cargo dependency/incremental caches.

### Prepared inputs (SHA-256)

| Input | SHA-256 |
| --- | --- |
| `apps/rustui-playground/src/stories/sidebar_gallery.rs` | `c5e57f62e340495b86a23690f9221aad36c3f98351828b2bdc72dfcd088cf02c` |
| `crates/rustui/src/components/navigation/sidebar.rs` | `b8f6ac76617a3ec9c7a201bf04ebaa2e3c4774c6b617d21dbdf5e67aa87c14fa` |
| `apps/rustui-playground/src/lib.rs` | `c7f4eb449f2f1c768f220c95af7a81288bffd2a61c677b7bf00e4b238579876b` |
| `apps/rustui-playground/src/catalog.rs` | `dbea784769e14210ef3f6dd5197112a1c515384f8efcc6fc2fd4f59eb3cbf807` |
| `apps/rustui-playground/src/stories/mod.rs` | `233717193b798488ffd10e06941bfa4d8316559d078ef6335846b1c46c4a3f84` |
| `apps/rustui-playground/src/stories/locale_selector.rs` | `d9dc31e61be19d12a8b5a18a97936415b1f4213a7cca481c8b849cbdd5016f33` |
| `apps/rustui-playground/src/stories/alert_gallery.rs` | `62392b0ab0ef3c2d5079dd98870320154ecf7e6aa6996111555745b117b774c1` |
| `apps/rustui-playground/locales/en/lab.ftl` | `b0745ced2cdd01c17c6778cd1297bd41c7cdd56aaba49b59289f62904c7ed0bf` |
| `apps/rustui-playground/locales/fr/lab.ftl` | `c56d3c6d481fd26b832822dad94f72ba639316dde8d88ec4ceebc4d4adbd4a6d` |
| `apps/rustui-playground/locales/ar/lab.ftl` | `705a51cdc7f09a3488d2dc7a322c2683ce2dcbe531c715ad6503f64ea8b29911` |
| N2 prebuild executable | `7db471997a1f86eee9f19315da2122c7857b1a5bcb3dd8c57fb209734792f570` |
| Earlier M417 evidence (partial, stale for this fixture) | `6bd91ae10ee951a5a252780f784fd94fe87053220c5881ff99eb04845db538af` |

The source package script and plist hashes are respectively
`4d06bb6ee90b974b1d2c86d552fa164545d9722fb33c6c09c7261e17fec9cec4` and
`6fdbe837e388c3c1ab715275ab398d226c35b54e3a784e71f55911a5f13ef6f3`.
