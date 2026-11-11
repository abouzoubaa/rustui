#!/usr/bin/env bash
# Source/build checks only. Native/browser/AX/performance acceptance is separate.
set -euo pipefail
repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"
. "$repository_root/scripts/pinned-toolchain.sh"

verify_tree_accessibility_series() {
  python3 scripts/apply_gpui_tree_accessibility_series.py --verify
  python3 scripts/apply_gpui_tree_aria_disabled_patch.py --check
}

# Exercise the actual dependency-free Web edit planner on the host toolchain.
# GPUI/Wasm compilation and browser acceptance remain separate checks.
verify_web_ime_edit_plan() (
  binary_dir="$repository_root/target/headstart/verification/web-ime-edit-plan"
  mkdir -p "$binary_dir"
  test_binary="$binary_dir/ime_edit_tests"
  trap 'rm -f "$test_binary"' EXIT
  pinned_rustc --edition 2024 --test \
    vendor/gpui-pre-web-0.3.9/src/ime_edit.rs -o "$test_binary"
  "$test_binary"
)

case "${1:-}" in
  source)
    python3 scripts/check_dependency_notices_freshness.py
    # The popup patch shares macOS source files with the Kit 0.8 stack. Reverse
    # patches before cycling that stack, then restore them in dependency order.
    popup_child_teardown_armed=0
    popup_patch_armed=0
    restore_popup_patch() {
      result=$?
      trap - EXIT
      if [ "$popup_patch_armed" -eq 1 ]; then
        if ! python3 scripts/apply_gpui_macos_anchored_popup_patch.py; then
          result=1
        else
          popup_patch_armed=0
        fi
      fi
      if [ "$popup_child_teardown_armed" -eq 1 ]; then
        if ! python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py; then
          result=1
        fi
      fi
      exit "$result"
    }
    trap restore_popup_patch EXIT
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py --reverse --check
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py --reverse
    popup_child_teardown_armed=1
    python3 scripts/apply_gpui_macos_anchored_popup_patch.py --reverse --check
    python3 scripts/apply_gpui_macos_anchored_popup_patch.py --reverse
    popup_patch_armed=1
    python3 scripts/apply_gpui_kit_08_stack.py --reverse
    python3 scripts/apply_gpui_kit_08_stack.py --apply
    python3 scripts/apply_gpui_kit_08_stack.py --check
    python3 scripts/apply_gpui_macos_anchored_popup_patch.py
    popup_patch_armed=0
    python3 scripts/apply_gpui_macos_anchored_popup_patch.py --verify
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py
    popup_child_teardown_armed=0
    trap - EXIT
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py --verify
    python3 scripts/verify_owned_gpui_sources.py
    python3 - <<'PY'
import unittest

legacy_gpui_snapshot_test_modules = {
    "test_apply_block_stack_symbol_patch",
    "test_apply_gpui_base_slider_endpoint",
    "test_apply_gpui_base_slider_first_move",
    "test_apply_gpui_base_slider_snap_position",
    "test_apply_gpui_base_slider_zero_track",
    "test_apply_gpui_calendar_heading_patch",
    "test_apply_gpui_calendar_keyboard_patch",
    "test_apply_gpui_calendar_rtl_patch",
    "test_apply_gpui_date_picker_rtl_patch",
    "test_apply_gpui_select_popup_rtl_patch",
    "test_apply_gpui_select_rtl_patch",
    "test_apply_gpui_tree_accessibility_series",
    "test_cosmic_bidi_patch",
    "test_g10_grid_accessibility_patches",
    "test_gpui_base_calendar_date_limits",
    "test_gpui_base_disabled_semantic_roots",
    "test_gpui_calendar_locale_patch",
    "test_gpui_date_picker_patch",
    "test_gpui_input_placeholder_color_patch",
    "test_gpui_inert_focus_scope_patch",
    "test_gpui_macos_focused_element_patch",
    "test_gpui_modal_focus_recovery_patch",
    "test_gpui_row_accessibility_proposal",
    "test_gpui_select_title_patch",
    "test_gpui_tree_accessibility_series",
    "test_gpui_validation_grid_a11y_patch",
    "test_gpui_toolbar_rtl",
    "test_gpui_web_semantic_source",
    "test_gpui_web_semantic_adapter",
    "test_apply_gpui_web_combobox_enter_patch",
    "test_gpui_web_noopener_patch",
    "test_gpui_web_syntax_patch",
    "test_p1_approved_patches",
    "test_p2_dialog_patch",
    "test_p2_list_patch",
    "test_p2_mouse_patch",
    "test_p2_sheet_patch",
    "test_p2_tree_patch",
    "test_p2b_slider_patches",
    "test_r15_disabled_patches",
    "test_r15b_dropdown_patches",
}
legacy_theme_test = "test_verify_theme_adapter_coverage.RendererCoverageTests.test_menu_theme_uses_activation_and_upstream_popover"

def active_snapshot_tests(suite):
    filtered = unittest.TestSuite()
    for test in suite:
        if isinstance(test, unittest.TestSuite):
            filtered.addTests(active_snapshot_tests(test))
        elif (
            test.id().split(".", 1)[0] not in legacy_gpui_snapshot_test_modules
            and not (
                test.id().startswith("unittest.loader._FailedTest.")
                and test.id().rsplit(".", 1)[-1] in legacy_gpui_snapshot_test_modules
            )
            and test.id() != legacy_theme_test
        ):
            filtered.addTest(test)
    return filtered

tests = active_snapshot_tests(
    unittest.defaultTestLoader.discover("scripts/tests", pattern="test_*.py")
)
result = unittest.TextTestRunner(verbosity=2).run(tests)
raise SystemExit(not result.wasSuccessful())
PY
    sh scripts/tests/test_package_playground_app.sh
    sh scripts/tests/test_verify_ci_checkout.sh
    python3 scripts/validate_locales.py
    python3 scripts/validate_themes.py --plans-only
    python3 scripts/verify_theme_adapter_coverage.py
    ;;
  macos)
    verify_tree_accessibility_series
    python3 scripts/apply_gpui_base_button_disabled_patch.py --verify
    python3 scripts/apply_gpui_base_focus_trap_accessibility_v1_patch.py --verify
    python3 scripts/verify_r15b_popup_menu_patch_stack.py
    python3 scripts/apply_r15b_menu_item_disabled_v1_patch.py --verify
    python3 scripts/apply_r15b_dropdown_menu_refresh_v1_patch.py --verify
    python3 scripts/apply_gpui_component_sheet_close_accessibility_v2_patch.py --verify
    python3 scripts/verify_r15_tree_patch_stack.py
    python3 scripts/apply_gpui_primary_mousedown_click_v2_patch.py --verify
    python3 scripts/apply_gpui_selectable_list_semantics_v2_patch.py --verify
    python3 scripts/apply_gpui_component_tracked_dialog_focus_v2_patch.py --verify
    python3 scripts/apply_gpui_base_resize_first_move.py --verify
    python3 scripts/apply_gpui_base_slider_set_value_v2_patch.py --verify
    python3 scripts/apply_gpui_component_slider_accessibility_label_v2_patch.py --verify
    python3 scripts/verify_g10_grid_debug_patch_stack.py
    python3 scripts/apply_r15_button_disabled_focus_v1_patch.py --verify
    python3 scripts/apply_r15_breadcrumb_disabled_v1_patch.py --verify
    python3 scripts/apply_r15_sidebar_disabled_v1_patch.py --verify
    python3 scripts/verify_gpui_base_slider_patch_stack.py
    python3 scripts/apply_block_stack_symbol_patch.py --verify
    python3 scripts/apply_gpui_component_calendar_locale_patch.py --check
    python3 scripts/apply_gpui_input_placeholder_color_patch.py --check
    python3 scripts/verify_gpui_calendar_patch_stack.py
    python3 scripts/apply_gpui_component_select_title_patch.py --check
    python3 scripts/apply_gpui_select_rtl_patch.py --verify
    python3 scripts/apply_gpui_select_popup_rtl_patch.py --verify
    python3 scripts/apply_gpui_component_web_syntax_patch.py --check
    python3 scripts/apply_gpui_component_web_syntax_runtime_patch.py --check
    python3 scripts/apply_cosmic_bidi_initial_span_patch.py --check
    python3 scripts/verify_gpui_web_patch_stack.py --native-only
    python3 scripts/apply_gpui_macos_focused_element_patch.py --verify
    python3 scripts/apply_gpui_modal_focus_recovery_patch.py --verify
    python3 scripts/apply_gpui_inert_focus_scope_patch.py --verify
    python3 scripts/apply_gpui_validation_grid_a11y_patch.py --verify
    python3 scripts/apply_g10_parent_node_id_v1_patch.py --verify
    # Native anchored popups are an approved, opt-in backend capability.
    popup_child_teardown_armed=0
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py --reverse --check
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py --reverse
    popup_child_teardown_armed=1
    restore_popup_child_teardown() {
      result=$?
      trap - EXIT
      if [ "$popup_child_teardown_armed" -eq 1 ]; then
        if ! python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py; then
          result=1
        fi
      fi
      exit "$result"
    }
    trap restore_popup_child_teardown EXIT
    python3 scripts/apply_gpui_macos_anchored_popup_patch.py --verify
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py
    popup_child_teardown_armed=0
    trap - EXIT
    python3 scripts/apply_gpui_macos_popup_child_teardown_patch.py --verify
    if [[ "$(uname -s)" != Darwin ]]; then
      echo 'The macos release check requires a macOS host.' >&2
      exit 2
    fi
    pinned_cargo fmt --all -- --check
    python3 scripts/verify_dependency_boundary.py
    ./scripts/cargo-headstart check --workspace --all-targets --profile headstart --locked --offline
    ./scripts/cargo-headstart check --manifest-path examples/external-consumer/Cargo.toml --profile headstart --locked --offline
    ./scripts/cargo-headstart test --workspace --lib --profile headstart --locked --offline
    ./scripts/cargo-headstart test --workspace --test '*' --profile headstart --locked --offline
    ./scripts/cargo-headstart test -p rustui-macros --doc --profile headstart --locked --offline
    evidence="$repository_root/target/headstart/evidence/release-checks"
    mkdir -p "$evidence"
    ./scripts/cargo-headstart metadata --format-version 1 --locked --offline > "$evidence/native-metadata.json"
    RUSTUI_REMAP_BUILD_PATHS=1 ./scripts/cargo-headstart build -p rustui-playground --release --locked --offline \
      --message-format=json > "$evidence/native-build.jsonl"
  if LC_ALL=C grep -aFq -- "$repository_root" "$repository_root/target/headstart/release/rustui-playground"; then
    echo 'native release binary embeds its build checkout path' >&2
    exit 1
  fi
    notices="$repository_root/target/headstart/evidence/native-dependency-notices-checked-$(date -u +%Y%m%dT%H%M%SZ)-$$"
    cargo_home="${CARGO_HOME:-$HOME/.cargo}"
    python3 scripts/collect_dependency_notices.py \
      --metadata "$evidence/native-metadata.json" \
      --build-json "$evidence/native-build.jsonl" \
      --repo-root "$repository_root" \
      --source-root "$cargo_home/registry/src" \
      --source-root "$cargo_home/git/checkouts" \
      --supplement-manifest docs/release/notice-supplements/manifest.json \
      --supplement-root docs/release/notice-supplements \
      --fail-on-unresolved-gaps \
      --output "$notices"
    binary="$(python3 - "$evidence/native-build.jsonl" <<'PY'
import json, sys
from pathlib import Path
executables = set()
for line in Path(sys.argv[1]).read_text().splitlines():
    message = json.loads(line)
    if (message.get('reason') == 'compiler-artifact'
            and message.get('target', {}).get('name') == 'rustui-playground'
            and message.get('executable')):
        executables.add(message['executable'])
if len(executables) != 1:
    sys.exit('Expected exactly one Playground executable in Cargo artifact evidence')
print(executables.pop())
PY
)"
    RUSTUI_PLAYGROUND_BINARY="$binary" \
    RUSTUI_RELEASE_NOTICES_REQUIRED=1 \
    RUSTUI_RELEASE_NOTICES_DIR="$notices" \
      apps/rustui-playground/package-app.sh --replace
    ;;
  web)
    verify_tree_accessibility_series
    python3 scripts/apply_block_stack_symbol_patch.py --verify
    python3 scripts/apply_gpui_component_calendar_locale_patch.py --check
    python3 scripts/apply_gpui_input_placeholder_color_patch.py --check
    python3 scripts/verify_gpui_calendar_patch_stack.py
    python3 scripts/apply_gpui_component_select_title_patch.py --check
    python3 scripts/apply_gpui_select_rtl_patch.py --verify
    python3 scripts/apply_gpui_select_popup_rtl_patch.py --verify
    python3 scripts/apply_gpui_web_noopener_patch.py --check
    python3 scripts/apply_gpui_component_web_syntax_patch.py --check
    python3 scripts/apply_gpui_component_web_syntax_runtime_patch.py --check
    python3 scripts/apply_cosmic_bidi_initial_span_patch.py --check
    python3 -m unittest discover -s scripts/tests -p 'test_apply_gpui_web_*.py'
    python3 -m unittest discover -s scripts/tests -p 'test_cosmic_bidi_patch.py'
    python3 -m unittest discover -s scripts/tests -p 'test_gpui_web_*.py'
    python3 -m unittest discover -s scripts/tests -p 'test_manifest_playground_web.py'
    python3 -m unittest discover -s scripts/tests -p 'test_provision_ci_wasm_bindgen.py'
    node --test scripts/tests/test_gpui_web_semantic_dom.mjs
    node --test apps/rustui-playground/web/tests/preview-transport.test.mjs
    node --test apps/rustui-playground/web/tests/theme-reload.test.mjs
    sh scripts/tests/test_build_playground_web.sh
    verify_web_ime_edit_plan
    python3 scripts/verify_gpui_web_patch_stack.py
    RUSTUI_HEADSTART_ROOT="${RUSTUI_WASM_HEADSTART_ROOT:-$repository_root/target/headstart/wasm-root}" \
    CARGO_TARGET_DIR="${RUSTUI_WASM_TARGET_DIR:-$repository_root/target/headstart/x1a-wasm}" \
      ./scripts/build-playground-web.sh
    ;;
  *)
    echo 'Usage: ./scripts/verify-release.sh source|macos|web' >&2
    exit 2
    ;;
esac
