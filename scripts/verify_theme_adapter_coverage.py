#!/usr/bin/env python3
"""Static theme wiring checks; source markers are not visual acceptance."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
THEME_COLORS = ROOT / "vendor/gpui-component-0.7.3/src/theme/theme_color.rs"
ADAPTER = ROOT / "crates/rustui/src/adapters/themes.rs"
REGISTRY = ROOT / "crates/rustui/src/theme/registry.rs"
MACOS = ROOT / "themes/macos27.json"

# Renderer files must query the corresponding JSON component record. Merely
# naming a component in the coverage declaration does not consume its rules.
STYLE_RENDERERS = {
    "button": "components/basic/button.rs",
    "link": "components/basic/link.rs",
    "text": "components/basic/text.rs",
    "label": "components/basic/label.rs",
    "checkbox": "components/input/checkbox.rs",
    "divider": "components/basic/divider.rs",
    "icon": "components/basic/icon.rs",
    "image": "components/basic/image.rs",
    "slider": "components/input/slider.rs",
    "toggle": "components/input/toggle.rs",
    "picker": "components/input/picker.rs",
    "stepper": "components/input/stepper.rs",
    "text_field": "components/input/text_field.rs",
    "search_field": "components/input/text_field.rs",
    "radio_button": "components/input/radio.rs",
    "combo_box": "components/input/combo_box.rs",
    "color_picker": "components/input/color_picker.rs",
    "disclosure_control": "components/navigation/disclosure.rs",
    "menu": "components/overlay/menu.rs",
    "notification": "components/overlay/toast.rs",
    "table": "components/data/table.rs",
    "outline_view": "components/data/tree.rs",
    "scroll_view": "layout/scroll_view.rs",
    "menu_item": "components/overlay/menu.rs",
}

PROFILE_RENDERERS = {
    "toolbar": ("components/navigation/toolbar.rs", "shell_metrics(&theme.profile)"),
    "sidebar": ("components/navigation/sidebar.rs", "theme.profile.dimensions.sidebar_width"),
    "tab_bar": ("components/navigation/tab_view.rs", "theme.profile.dimensions.control_height"),
}


def check_renderer_calls(source_overrides: dict[str, str] | None = None) -> list[str]:
    theme = json.loads(MACOS.read_text(encoding="utf-8"))
    registry = REGISTRY.read_text(encoding="utf-8")
    mapped_block = registry.split("const MAPPED_COMPONENTS:", 1)[1].split("];", 1)[0]
    mapped = set(re.findall(r'"([a-z_]+)"', mapped_block))
    styled = {name for name in mapped if theme["components"][name].get("styles")}
    source_overrides = source_overrides or {}
    errors = [f"No renderer registered for {name}" for name in sorted(styled - STYLE_RENDERERS.keys())]
    for name, relative_path in STYLE_RENDERERS.items():
        if name not in styled:
            errors.append(f"Stale renderer mapping for {name}")
            continue
        source = source_overrides.get(relative_path)
        if source is None:
            source = (ROOT / "crates/rustui/src" / relative_path).read_text(encoding="utf-8")
        source = _without_test_modules(source)
        if name == "button":
            queried = "context.button_theme()" in source
        elif name in {"text_field", "search_field"}:
            part = "body" if name == "text_field" else "list_search"
            queried = (
                f'("{name}", "{part}")' in source
                and "required_style_with_artwork(context, component, part" in source
            )
        else:
            pattern = (
                r"required_(?:app_)?style(?:_with_artwork)?\s*\(\s*"
                r"(?:context|cx)\s*,\s*\"" + re.escape(name) + r"\""
            )
            queried = re.search(pattern, source) is not None
        if not queried:
            errors.append(f"{name} is not queried by {relative_path}")
        if name != "button" and not (
            "styled_element(" in source
            or "styled_container(" in source
            or re.search(r"\bstyle\s*\.\s*(colors|lengths|typography|record_metadata)", source)
        ):
            errors.append(f"{name} has no resolved-style consumer in {relative_path}")
    for name, (relative_path, profile_use) in PROFILE_RENDERERS.items():
        if name not in mapped or theme["components"][name].get("styles"):
            errors.append(f"{name} profile fallback no longer matches its theme record")
            continue
        source = (ROOT / "crates/rustui/src" / relative_path).read_text(encoding="utf-8")
        if profile_use not in source:
            errors.append(f"{name} does not consume its profile metric in {relative_path}")
    return errors


CONTROL_CONTRACTS = {
    "checkbox": {
        "path": "components/input/checkbox.rs", "part": "body",
        "roles": {"checked": "primary", "unchecked": "border", "foreground": "foreground"},
        "properties": {
            "background": (r'style\s*\.\s*colors\s*\[\s*"background"\s*\]', r'\.bg\s*\(\s*if\s+checked\s*\{\s*accent\s*\}', r'\.border_color\s*\(\s*if\s+checked\s*\{\s*accent\s*\}'),
            "foreground": (r'style\s*\.\s*colors\s*\[\s*"foreground"\s*\]', r'\.text_color\s*\(\s*foreground\s*\)'),
            "typography": (r'&\s*style\s*\.\s*typography', r'\.font_family\s*\(', r'\.text_size\s*\(', r'\.font_weight\s*\('),
            "vector_path": (r'rectangular_artwork_size\s*\(\s*&style\s*\)', r'\.w\s*\(\s*px\(width\)\s*\)', r'\.h\s*\(\s*px\(height\)\s*\)'),
        },
        "roles_evidence": {"unchecked": (r'appearance_color\s*\(\s*&active\.profile\s*,\s*active\.mode\s*,\s*"border"',)},
    },
    "radio_button": {
        "path": "components/input/radio.rs", "part": "body",
        "roles": {"selected": "primary", "unselected": "border", "foreground": "foreground"},
        "properties": {
            "width": (r'style\s*\.\s*lengths\s*\[\s*"width"\s*\]', r'\.w\s*\(\s*px\(width\)\s*\)'),
            "height": (r'style\s*\.\s*lengths\s*\[\s*"height"\s*\]', r'\.h\s*\(\s*px\(height\)\s*\)'),
            "corner_radius": (r'style\s*\.\s*lengths\s*\[\s*"corner_radius"\s*\]', r'\.rounded\s*\(\s*px\(radius\)\s*\)'),
            "background": (r'style\s*\.\s*colors\s*\[\s*"background"\s*\]', r'\.bg\s*\(\s*if\s+selected\s*\{\s*accent\s*\}', r'\.border_color\s*\(\s*if\s+selected\s*\{\s*accent\s*\}'),
            "foreground": (r'style\s*\.\s*colors\s*\[\s*"foreground"\s*\]', r'\.bg\s*\(\s*foreground\s*\)', r'\.text_color\s*\(\s*color\s*\)'),
            "typography": (r'&\s*style\s*\.\s*typography', r'\.font_family\s*\(', r'\.text_size\s*\(', r'\.font_weight\s*\('),
            "vector_path": (r'rectangular_artwork_size\s*\(\s*&style\s*\)', r'\.w\s*\(\s*px\(width\)\s*\)', r'\.h\s*\(\s*px\(height\)\s*\)'),
        },
        # This declared dark appearance property is not currently applied by
        # the GPUI renderer; report it as a known limit, not consumed coverage.
        "known_unapplied": {"blend_mode": "GPUI radio renderer does not apply blend modes"},
        "roles_evidence": {"unselected": (r'appearance_color\s*\(\s*&active\.profile\s*,\s*active\.mode\s*,\s*"border"',)},
    },
    "slider": {
        "path": "components/input/slider.rs", "part": "sliders",
        "roles": {"track": "secondary", "thumb": "primary", "disabled": "muted_foreground"},
        "properties": {
            "width": (r'style\s*\.\s*lengths\s*\.\s*get\s*\(\s*"width"', r'thumb_width\s*,', r'\.w\s*\(\s*px\(self\.thumb_width\)\s*\)'),
            "height": (r'style\s*\.\s*lengths\s*\.\s*get\s*\(\s*"height"', r'thumb_height\s*,', r'\.h\s*\(\s*px\(self\.thumb_height\)\s*\)'),
        },
        "roles_evidence": {
            "track": (r'track_color\s*:\s*slider_track_color\s*\(', r'"track"', r'let\s+track\s*=\s*slider_role_color\s*\(\s*style\s*,\s*role', r'if\s+track\s*==\s*background', r'try_parse_color\s*\(\s*&colors\.secondary_text\s*\)', r'\.bg\s*\(\s*self\s*\.\s*track_color\s*\.\s*opacity', r'let\s+fill_color\s*=\s*if\s+self\.disabled\s*\{\s*self\.disabled_color.*?else\s*\{\s*self\.track_color', r'"secondary"\s*=>\s*&colors\.toolbar_background'),
            "thumb": (r'thumb_color\s*:\s*slider_role_color\s*\(', r'"thumb"', r'\.bg\s*\(\s*thumb_color\s*\)', r'let\s+thumb_color\s*=\s*if\s+self\.disabled\s*\{\s*self\.disabled_color.*?else\s*\{\s*self\.thumb_color', r'"primary"\s*=>\s*&colors\.accent'),
            "disabled": (r'disabled_color\s*:\s*slider_role_color\s*\(', r'"disabled"', r'self\.disabled_color', r'"muted_foreground"\s*=>\s*&colors\.secondary_text'),
        },
    },
    "toggle": {
        "path": "components/input/toggle.rs", "part": "body",
        "roles": {"on": "primary", "off": "secondary", "thumb": "foreground", "disabled": "muted_foreground"},
        "properties": {
            "width": (r'style\s*\.\s*lengths\s*\.\s*get\s*\(\s*"width"', r'width\s*,', r'\.w\s*\(\s*px\(self\.width\)\s*\)'),
            "height": (r'style\s*\.\s*lengths\s*\.\s*get\s*\(\s*"height"', r'height\s*,', r'\.h\s*\(\s*px\(self\.height\)\s*\)'),
            "corner_radius": (r'\.get\s*\(\s*"corner_radius"', r'radius\s*,', r'\.rounded\s*\(\s*px\(self\.radius\)\s*\)'),
            "background": (r'\.colors\s*\.\s*get\s*\(\s*"background"', r'checked_color\s*,', r'let\s+track_color\s*=\s*if\s+self\.checked\s*\{\s*self\.checked_color\s*\}\s*else\s*\{\s*self\.unchecked_color', r'\.bg\s*\(\s*if\s+self\.disabled'),
        },
        "roles_evidence": {
            "on": (r'\.colors\s*\.\s*get\s*\(\s*"background"', r'let\s+track_color\s*=\s*if\s+self\.checked\s*\{\s*self\.checked_color\s*\}', r'\.bg\s*\(\s*if\s+self\.disabled'),
            "off": (r'colors\.toolbar_background', r'unchecked_color\s*,', r'let\s+track_color\s*=\s*if\s+self\.checked\s*\{\s*self\.checked_color\s*\}\s*else\s*\{\s*self\.unchecked_color', r'\.bg\s*\(\s*if\s+self\.disabled'),
            "thumb": (r'colors\.primary_text', r'foreground_color\s*,', r'let\s+thumb_color\s*=\s*if\s+self\.disabled\s*\{\s*self\.disabled_thumb_color\s*\}\s*else\s*\{\s*self\.foreground_color', r'\.bg\s*\(\s*thumb_color\s*\)'),
            "disabled": (r'colors\.secondary_text', r'disabled_thumb_color', r'label_color\s*=\s*if\s+self\.disabled', r'\.text_color\s*\(\s*self\.label_color\s*\)'),
        },
    },
    "scroll_view": {
        "path": "layout/scroll_view.rs", "part": "scrollbar_vertical",
        "roles": {"thumb": "muted_foreground"},
        "properties": {
            "background": (r'style\s*\.\s*colors\s*\.\s*get\s*\(\s*"background"', r'track\s*\.\s*bg\s*\(\s*background\s*\)'),
            "border": (r'style\s*\.\s*colors\s*\.\s*get\s*\(\s*"border"', r'track\s*\.\s*border_color\s*\(\s*border\s*\)'),
            "corner_radius": (r'style\s*\.\s*lengths\s*\.\s*get\s*\(\s*"corner_radius"', r'\.radius\s*\(\s*px\(radius\)\s*\)'),
        },
        "known_unapplied": {
            "width": "macos27 source-artwork extent is not the scrollbar thumb thickness",
            "height": "macos27 source-artwork extent is not the viewport-sized scrollbar length",
        },
    },
}


def _without_test_modules(source: str) -> str:
    """Strip cfg(test) modules so fixtures cannot satisfy renderer checks."""
    marker = re.compile(r"(?m)^\s*#\[cfg\(test\)\]\s*\n\s*mod\s+\w+\s*\{")
    while match := marker.search(source):
        opening = source.find("{", match.start(), match.end())
        depth = 0
        quote = escaped = line_comment = False
        block_depth = 0
        i = opening
        while i < len(source):
            ch = source[i]
            nxt = source[i + 1] if i + 1 < len(source) else ""
            if line_comment:
                line_comment = ch != "\n"
            elif block_depth:
                if ch == "/" and nxt == "*":
                    block_depth += 1
                    i += 1
                elif ch == "*" and nxt == "/":
                    block_depth -= 1
                    i += 1
            elif quote:
                if escaped:
                    escaped = False
                elif ch == "\\":
                    escaped = True
                elif ch == '"':
                    quote = False
            elif ch == "/" and nxt == "/":
                line_comment = True
                i += 1
            elif ch == "/" and nxt == "*":
                block_depth = 1
                i += 1
            elif ch == '"':
                quote = True
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    source = source[:match.start()] + source[i + 1:]
                    break
            i += 1
        else:
            return source
    return re.sub(r"(?m)//[^\n]*$", "", source)


def check_control_style_consumption(
    components: dict | None = None,
    source_overrides: dict[str, str] | None = None,
) -> list[str]:
    """Require concrete style reads/applications for the four themed controls.

    This is lexical evidence, not Rust AST/data-flow analysis: it cannot prove
    reads and paint operations execute in the same branch or prove rendered
    pixels. GUI acceptance is still required. The test-module stripper handles
    ordinary strings and comments, not the entire Rust lexical grammar.
    """
    if components is None:
        components = json.loads(MACOS.read_text(encoding="utf-8"))["components"]
    source_overrides = source_overrides or {}
    errors: list[str] = []
    for name, contract in CONTROL_CONTRACTS.items():
        record = components.get(name)
        if record is None:
            errors.append(f"{name} missing from macos27 component records")
            continue
        if record.get("theme_roles") != contract["roles"]:
            errors.append(f"{name} theme_roles differ from the checked role contract")
        rules = [rule for rule in record.get("styles", []) if rule.get("part") == contract["part"]]
        if not rules:
            errors.append(f"{name} has no macos27 rules for part {contract['part']}")
        rule_properties = [set(rule.get("style", {}).get("properties", {})) for rule in rules]
        required = set(contract["properties"])
        unapplied = set(contract.get("known_unapplied", {}))
        declared = set().union(*rule_properties) if rule_properties else set()
        for prop in required - declared:
            errors.append(f"{name} macos27 rules do not declare required property {prop}")
        for prop in declared - required - unapplied:
            errors.append(f"{name} has no consumption contract for macos27 property {prop}")
        for prop in required:
            if any(prop not in properties for properties in rule_properties):
                errors.append(f"{name}.{prop} is absent from one or more appearance rules")

        rel = contract["path"]
        try:
            raw = source_overrides.get(rel) or (ROOT / "crates/rustui/src" / rel).read_text(encoding="utf-8")
        except OSError:
            errors.append(f"{name} renderer is missing: {rel}")
            continue
        source = _without_test_modules(raw)
        if name == "scroll_view":
            call = r'required_style\s*\(\s*context\s*,\s*"scroll_view"\s*,\s*part\s*,'
            if not re.search(call, source, re.S) or not all(
                f'"{part}"' in source for part in ("scrollbar_vertical", "scrollbar_horizontal")
            ):
                errors.append("scroll_view renderer does not request both macos27 scrollbar track styles")
        else:
            call = r'(?:required_style|required_style_with_artwork)\s*\(\s*context\s*,\s*"' + re.escape(name) + r'"\s*,\s*"' + re.escape(contract["part"]) + r'"'
            if not re.search(call, source, re.S):
                errors.append(f"{name} renderer does not request its macos27 component/part style")
        for prop, markers in contract["properties"].items():
            if any(not re.search(marker, source, re.S) for marker in markers):
                errors.append(f"{name}.{prop} is not proven read and applied by source markers")
        for role, markers in contract.get("roles_evidence", {}).items():
            if any(not re.search(marker, source, re.S) for marker in markers):
                errors.append(f"{name} role `{role}` is not proven resolved and applied by source markers")
        if re.search(r"\bcx\s*\.\s*theme\s*\(", source):
            errors.append(f"{name} renderer reads GPUI default theme values")
    return errors


def main() -> int:
    theme_source = THEME_COLORS.read_text(encoding="utf-8")
    adapter_source = ADAPTER.read_text(encoding="utf-8")
    match = re.search(r"pub struct ThemeColor\s*\{(?P<body>.*?)\n\}", theme_source, re.S)
    if match is None:
        print(f"Could not find ThemeColor in {THEME_COLORS}", file=sys.stderr)
        return 2

    roles = set(re.findall(r"^\s*pub (\w+): Hsla,", match.group("body"), re.M))
    mapped = set(re.findall(r"\bcolors\.(\w+)\s*=", adapter_source))
    missing = sorted(roles - mapped)
    extra = sorted(mapped - roles)
    renderer_errors = check_renderer_calls() + check_control_style_consumption()
    if missing or extra or renderer_errors:
        if missing:
            print("Unmapped GPUI Components theme roles:", ", ".join(missing), file=sys.stderr)
        if extra:
            print("Unknown GPUI Components theme assignments:", ", ".join(extra), file=sys.stderr)
        for error in renderer_errors:
            print("Runtime theme mapping:", error, file=sys.stderr)
        return 1

    print(f"Mapped all {len(roles)} GPUI Components ThemeColor roles.")
    print(f"Found {len(STYLE_RENDERERS)} styled and {len(PROFILE_RENDERERS)} profile renderer paths.")
    print("Static macos27 source checks cover checkbox, radio_button, slider, and toggle; this is not visual acceptance.")
    print("Artwork checks verify checkbox/radio path bounds are read for sizing; they do not verify rendered path pixels.")
    print("ScrollView track border colors are checked against the pinned ScrollbarTrackStyle API; native visual acceptance remains separate.")
    print("Known unapplied macos27 properties: scroll_view.width/height are source-artwork extents; the pinned API derives thumb length from viewport geometry.")
    print("Known unapplied macos27 property: radio_button.blend_mode (GPUI renderer has no blend-mode application).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
