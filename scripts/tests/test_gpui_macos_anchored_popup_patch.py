from __future__ import annotations

import json
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from scripts.apply_gpui_macos_anchored_popup_patch import (
    PATCH,
    POSTIMAGES,
    PREIMAGES,
    PatchError,
    apply_patch,
    sha256,
    verify_applied,
)
from scripts.apply_gpui_macos_popup_child_teardown_patch import (
    apply_patch as apply_child_teardown_patch,
)


ROOT = Path(__file__).resolve().parents[2]


def extract_function(source: str, name: str) -> str:
    declaration = re.compile(rf"(?m)^\s*fn\s+{re.escape(name)}\s*\(")
    matches = list(declaration.finditer(source))
    if len(matches) != 1:
        raise ValueError(f"expected one exact function named {name}, found {len(matches)}")
    start = matches[0].start()
    opening = source.index("{", matches[0].end())
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise ValueError(f"unterminated function {name}")


class AnchoredPopupPatchTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="rustui-popup-patch-test-")
        self.root = Path(self.temp.name)
        for relative in PREIMAGES:
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, destination)
        self.assertEqual(
            apply_child_teardown_patch(self.root, reverse=True),
            "reversed",
        )
        state = apply_patch(self.root, check_only=True)
        if state == "already-applied":
            self.assertEqual(apply_patch(self.root, reverse=True), "reversed")
        else:
            self.assertEqual(state, "verified")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_exact_patch_applies_and_is_idempotent(self) -> None:
        self.assertEqual(apply_patch(self.root), "applied")
        self.assertEqual(
            {relative: sha256(self.root / relative) for relative in PREIMAGES},
            POSTIMAGES,
        )
        self.assertEqual(apply_patch(self.root), "already-applied")

    def test_check_mode_does_not_change_sources(self) -> None:
        before = {relative: sha256(self.root / relative) for relative in PREIMAGES}
        self.assertEqual(apply_patch(self.root, check_only=True), "verified")
        self.assertEqual(before, {relative: sha256(self.root / relative) for relative in PREIMAGES})

    def test_screen_conversion_uses_anchor_views_native_window(self) -> None:
        patch = PATCH.read_text(encoding="utf-8")
        self.assertIn("let anchor_window: *mut AnyObject = msg_send![parent_view, window];", patch)
        self.assertIn("msg_send![anchor_window, convertRectToScreen: anchor_in_window]", patch)

    def test_native_screen_placement_uses_visible_frame_helpers(self) -> None:
        source = (ROOT / "crates/rustui-gpui-macos/src/window.rs").read_text(encoding="utf-8")
        frame_start = source.index("fn anchored_popup_frame(")
        frame_end = source.index("\nfn popup_screen_index(", frame_start)
        frame = source[frame_start:frame_end]
        self.assertIn("NSScreen::mainScreen(marker).expect(\"main screen exists\")", frame)
        self.assertIn("popup_screen_index(anchor_center, &visible_frames, main_screen_index)", frame)
        self.assertIn("popup_frame_for_visible_screen(anchor, visible, popup_size, options)", frame)

    def test_logical_anchor_points_reach_appkit_without_backing_scale(self) -> None:
        source = (ROOT / "crates/rustui-gpui-macos/src/window.rs").read_text(encoding="utf-8")
        helper_start = source.index("fn logical_anchor_rect_to_appkit_points(")
        helper_end = source.index("\nfn popup_anchor_rect_in_view(", helper_start)
        helper = source[helper_start:helper_end]
        frame_start = source.index("fn anchored_popup_frame(")
        frame_end = source.index("\nfn popup_anchor_x(", frame_start)
        frame = source[frame_start:frame_end]

        self.assertIn("anchor.origin.x.as_f32() as f64", helper)
        self.assertIn("anchor.size.height.as_f32() as f64", helper)
        self.assertNotIn("scale_factor", helper)
        self.assertIn("logical_anchor_rect_to_appkit_points(options.anchor_rect)", frame)
        self.assertIn("convertRect: anchor, toView:", frame)
        self.assertIn("convertRectToScreen: anchor_in_window", frame)

    def test_fractional_logical_and_view_bounds_placement_assertions_are_in_patch(self) -> None:
        patch = PATCH.read_text(encoding="utf-8")
        self.assertIn("fractional_logical_anchor_pixels_convert_to_appkit_points_without_scaling", patch)
        self.assertIn("assert_eq!(flipped.origin.y, 50.75);", patch)
        self.assertIn("assert_eq!(unflipped.origin.y, 182.25);", patch)

    def test_rust_logical_anchor_geometry_functions(self) -> None:
        source = (ROOT / "crates/rustui-gpui-macos/src/window.rs").read_text(encoding="utf-8")

        geometry = """#[derive(Clone, Copy)]
struct Pixels(f32);
impl Pixels { fn as_f32(self) -> f32 { self.0 } }
#[derive(Clone, Copy)]
struct Point<T> { x: T, y: T }
#[derive(Clone, Copy)]
struct Size<T> { width: T, height: T }
#[derive(Clone, Copy)]
struct Bounds<T> { origin: Point<T>, size: Size<T> }
impl<T> Bounds<T> { fn new(origin: Point<T>, size: Size<T>) -> Self { Self { origin, size } } }
fn point(x: Pixels, y: Pixels) -> Point<Pixels> { Point { x, y } }
fn size(width: Pixels, height: Pixels) -> Size<Pixels> { Size { width, height } }
fn px(value: f32) -> Pixels { Pixels(value) }
#[derive(Clone, Copy)]
struct NSPoint { x: f64, y: f64 }
impl NSPoint { fn new(x: f64, y: f64) -> Self { Self { x, y } } }
#[derive(Clone, Copy)]
struct NSSize { width: f64, height: f64 }
impl NSSize { fn new(width: f64, height: f64) -> Self { Self { width, height } } }
#[derive(Clone, Copy)]
struct NSRect { origin: NSPoint, size: NSSize }
impl NSRect { fn new(origin: NSPoint, size: NSSize) -> Self { Self { origin, size } } }
"""
        test = extract_function(
            source,
            "fractional_logical_anchor_pixels_convert_to_appkit_points_without_scaling",
        )
        rust_source = "\n".join(
            [
                geometry,
                extract_function(source, "logical_anchor_rect_to_appkit_points"),
                extract_function(source, "popup_anchor_rect_in_view"),
                "#[test]",
                test,
            ]
        )
        with tempfile.TemporaryDirectory(prefix="rustui-popup-geometry-test-") as directory:
            fixture = Path(directory)
            source_path = fixture / "popup_geometry.rs"
            binary_path = fixture / "popup_geometry_tests"
            source_path.write_text(rust_source, encoding="utf-8")
            compiled = subprocess.run(
                ["rustup", "run", "1.98.0", "rustc", "--test", str(source_path), "-o", str(binary_path)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(compiled.returncode, 0, compiled.stderr)
            executed = subprocess.run([str(binary_path)], capture_output=True, text=True, check=False)
            self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)
            self.assertIn("1 passed", executed.stdout)

    def test_rust_screen_selection_and_visible_frame_geometry(self) -> None:
        source = (ROOT / "crates/rustui-gpui-macos/src/window.rs").read_text(encoding="utf-8")

        geometry = """#[derive(Clone, Copy)]
struct Pixels(f32);
impl Pixels { fn as_f32(self) -> f32 { self.0 } }
#[derive(Clone, Copy)]
struct Point<T> { x: T, y: T }
#[derive(Clone, Copy)]
struct Size<T> { width: T, height: T }
impl<T> Size<T> { fn new(width: T, height: T) -> Self { Self { width, height } } }
#[derive(Clone, Copy)]
struct Bounds<T> { origin: Point<T>, size: Size<T> }
impl<T> Bounds<T> { fn new(origin: Point<T>, size: Size<T>) -> Self { Self { origin, size } } }
#[derive(Clone, Copy)]
struct NSPoint { x: f64, y: f64 }
impl NSPoint { fn new(x: f64, y: f64) -> Self { Self { x, y } } }
#[derive(Clone, Copy)]
struct NSSize { width: f64, height: f64 }
impl NSSize { fn new(width: f64, height: f64) -> Self { Self { width, height } } }
#[derive(Clone, Copy)]
struct NSRect { origin: NSPoint, size: NSSize }
impl NSRect { fn new(origin: NSPoint, size: NSSize) -> Self { Self { origin, size } } }
fn point(x: Pixels, y: Pixels) -> Point<Pixels> { Point { x, y } }
fn size(width: Pixels, height: Pixels) -> Size<Pixels> { Size { width, height } }
fn px(value: f32) -> Pixels { Pixels(value) }
#[derive(Clone, Copy)]
enum PopupAnchor { Center, Top, Bottom, Left, Right, TopLeft, BottomLeft, TopRight, BottomRight }
#[derive(Clone, Copy)]
enum PopupGravity { Center, Top, Bottom, Left, Right, TopLeft, BottomLeft, TopRight, BottomRight }
#[derive(Clone, Copy)]
struct PopupConstraintAdjustment(u8);
impl PopupConstraintAdjustment {
    const FLIP_X: Self = Self(1);
    const FLIP_Y: Self = Self(2);
    const SLIDE_X: Self = Self(4);
    const SLIDE_Y: Self = Self(8);
    const RESIZE_X: Self = Self(16);
    const RESIZE_Y: Self = Self(32);
    fn contains(self, other: Self) -> bool { self.0 & other.0 == other.0 }
}
impl std::ops::BitOr for PopupConstraintAdjustment {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { Self(self.0 | rhs.0) }
}
struct PopupOptions {
    parent: (),
    anchor_rect: Bounds<Pixels>,
    anchor: PopupAnchor,
    gravity: PopupGravity,
    constraint_adjustment: PopupConstraintAdjustment,
    offset: Point<Pixels>,
    grab: bool,
}
"""
        helpers = [
            "popup_screen_index",
            "popup_frame_for_visible_screen",
            "popup_anchor_x",
            "popup_anchor_y",
            "popup_origin",
            "popup_overflow_x",
            "popup_overflow_y",
            "flip_popup_gravity_x",
            "flip_popup_anchor_x",
            "flip_popup_gravity_y",
            "flip_popup_anchor_y",
        ]
        tests = [
            "screen_selection_handles_negative_stacked_origins_and_main_fallback",
        ]
        visible_frame_test = """#[test]
fn oversized_popup_resizes_to_negative_origin_visible_frame() {
    let options = PopupOptions {
        parent: (),
        anchor_rect: Bounds::new(point(px(-1910.0), px(900.0)), size(px(20.0), px(12.0))),
        anchor: PopupAnchor::BottomLeft,
        gravity: PopupGravity::BottomRight,
        constraint_adjustment: PopupConstraintAdjustment::FLIP_X
            | PopupConstraintAdjustment::FLIP_Y
            | PopupConstraintAdjustment::SLIDE_X
            | PopupConstraintAdjustment::SLIDE_Y
            | PopupConstraintAdjustment::RESIZE_X
            | PopupConstraintAdjustment::RESIZE_Y,
        offset: point(px(0.0), px(0.0)),
        grab: true,
    };
    let visible = NSRect::new(NSPoint::new(-1920.0, 23.0), NSSize::new(1920.0, 1057.0));
    let anchor = NSRect::new(NSPoint::new(-1910.0, 900.0), NSSize::new(20.0, 12.0));
    let (frame, popup_size) =
        popup_frame_for_visible_screen(anchor, visible, size(px(2400.0), px(1400.0)), &options);
    assert_eq!(frame.origin.x, -1920.0);
    assert_eq!(frame.origin.y, 23.0);
    assert_eq!(frame.size.width, 1920.0);
    assert_eq!(frame.size.height, 1057.0);
    assert_eq!(popup_size.width.as_f32(), 1920.0);
    assert_eq!(popup_size.height.as_f32(), 1057.0);
}
"""
        rust_source = "\n".join(
            [
                geometry,
                *(extract_function(source, name) for name in helpers),
                *("#[test]\n" + extract_function(source, name) for name in tests),
                visible_frame_test,
            ]
        )
        with tempfile.TemporaryDirectory(prefix="rustui-popup-screen-test-") as directory:
            fixture = Path(directory)
            source_path = fixture / "popup_screen.rs"
            binary_path = fixture / "popup_screen_tests"
            source_path.write_text(rust_source, encoding="utf-8")
            compiled = subprocess.run(
                ["rustup", "run", "1.98.0", "rustc", "--test", str(source_path), "-o", str(binary_path)],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(compiled.returncode, 0, compiled.stderr)
            executed = subprocess.run([str(binary_path)], capture_output=True, text=True, check=False)
            self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)
            self.assertIn("2 passed", executed.stdout)

    def test_extractor_rejects_prefix_matches_and_gpui_context_tests(self) -> None:
        source = """fn screen_selection_handles_negative_stacked_origins_and_main_fallback() { assert!(true); }
fn screen_selection_handles_negative_stacked_origins_and_main_fallback_extended() { panic!(); }
fn oversized_popup_resizes_to_negative_origin_visible_frame(cx: &mut gpui::TestAppContext) { panic!(); }
fn nested_grabbing_popups_dismiss_leaf_before_parent() { panic!(); }
"""
        extracted = extract_function(
            source,
            "screen_selection_handles_negative_stacked_origins_and_main_fallback",
        )
        self.assertEqual(
            extracted,
            "fn screen_selection_handles_negative_stacked_origins_and_main_fallback() { assert!(true); }",
        )
        for unintended in (
            "oversized_popup_resizes_to_negative_origin_visible_frame",
            "nested_grabbing_popups_dismiss_leaf_before_parent",
        ):
            self.assertNotIn(f"fn {unintended}", extracted)
        with self.assertRaisesRegex(ValueError, "found 0"):
            extract_function(source, "screen_selection_handles_negative_stacked_origins_and")

    def test_parent_teardown_closes_popup_children_without_unbounded_loop(self) -> None:
        patch = PATCH.read_text(encoding="utf-8")
        self.assertIn("for index in (0..child_count).rev()", patch)
        self.assertIn("removeChildWindow: child", patch)

    def test_open_popup_reposition_reaches_the_appkit_frame(self) -> None:
        patch = PATCH.read_text(encoding="utf-8")
        rustui = (ROOT / "crates/rustui/src/runtime/window.rs").read_text(encoding="utf-8")
        self.assertIn("fn set_anchored_popup_anchor_rect", patch)
        self.assertIn("observe_window_bounds", patch)
        self.assertIn("observe_window_display", rustui)
        self.assertIn("setFrame: frame display: Bool::YES", patch)
        self.assertIn("last_anchored_popup_anchor_rect", patch)
        self.assertNotIn("+        loop {", patch)

    def test_reverse_restores_exact_preimages(self) -> None:
        apply_patch(self.root)
        self.assertEqual(apply_patch(self.root, reverse=True), "reversed")
        self.assertEqual(
            {relative: sha256(self.root / relative) for relative in PREIMAGES},
            PREIMAGES,
        )

    def test_unknown_source_is_preserved_and_rejected(self) -> None:
        source = self.root / next(iter(PREIMAGES))
        source.write_bytes(source.read_bytes() + b"\nlocal change\n")
        before = source.read_bytes()
        with self.assertRaises(PatchError):
            apply_patch(self.root)
        self.assertEqual(source.read_bytes(), before)

    def test_approved_patch_manifest_binds_vendor_and_owned_images(self) -> None:
        manifest = json.loads((PATCH.parent / "gpui-macos-anchored-popup-kit08-v1.json").read_text())
        self.assertEqual(manifest["candidate"], PATCH.name)
        self.assertEqual(len(manifest["files"]), 12)
        self.assertTrue(any(path.startswith("vendor/") for path in manifest["files"]))
        self.assertTrue(any(path.startswith("crates/rustui-gpui-macos/") for path in manifest["files"]))

    def test_approved_patch_is_present(self) -> None:
        self.assertTrue(PATCH.is_file())

    def test_approved_patch_postimages_are_active(self) -> None:
        with tempfile.TemporaryDirectory(prefix="rustui-popup-verify-test-") as temporary:
            fixture = Path(temporary)
            for relative in PREIMAGES:
                target = fixture / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / relative, target)
            self.assertEqual(
                apply_child_teardown_patch(fixture, reverse=True),
                "reversed",
            )
            self.assertEqual(verify_applied(fixture), "verified")


if __name__ == "__main__":
    unittest.main()
