"""Normalize one exact keyboard-patched source in an isolated fixture."""
import importlib.util
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('calendar_keyboard_fixture', ROOT / 'scripts/apply_gpui_calendar_keyboard_patch.py')
keyboard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(keyboard)
RTL_SPEC = importlib.util.spec_from_file_location('calendar_rtl_fixture', ROOT / 'scripts/apply_gpui_calendar_rtl_patch.py')
rtl = importlib.util.module_from_spec(RTL_SPEC)
RTL_SPEC.loader.exec_module(rtl)
HEADING_SPEC = importlib.util.spec_from_file_location('calendar_heading_fixture', ROOT / 'scripts/apply_gpui_calendar_heading_patch.py')
heading = importlib.util.module_from_spec(HEADING_SPEC)
HEADING_SPEC.loader.exec_module(heading)


def reverse_heading_layer(fixture, relative):
    relative = Path(relative)
    if relative not in heading.IMAGES:
        return
    source = fixture / relative
    pre, post = heading.IMAGES[relative]
    current = heading.digest(source.read_bytes())
    if current == pre:
        return
    if current != post:
        raise ValueError(f'unexpected heading fixture image: {relative}')
    heading.run(ROOT, ROOT, '--verify')
    subprocess.run(['git', 'apply', '--reverse', '--whitespace=nowarn',
                    f'--include={relative}', str(ROOT / heading.PATCH)],
                   cwd=fixture, check=True, capture_output=True)
    if heading.digest(source.read_bytes()) != pre:
        raise ValueError(f'incorrect reversed heading fixture image: {relative}')


def reverse_rtl_layer(fixture, relative):
    relative = Path(relative)
    reverse_heading_layer(fixture, relative)
    source = fixture / relative
    pre, post = rtl.IMAGES[relative]
    current = rtl.digest(source.read_bytes())
    if current == pre:
        return
    if current != post:
        raise ValueError(f'unexpected RTL fixture image: {relative}')
    subprocess.run(['git', 'apply', '--reverse', '--whitespace=nowarn',
                    f'--include={relative}', str(ROOT / rtl.PATCH)],
                   cwd=fixture, check=True, capture_output=True)
    if rtl.digest(source.read_bytes()) != pre:
        raise ValueError(f'incorrect reversed RTL fixture image: {relative}')


def reverse_keyboard_layer(fixture, relative):
    relative = Path(relative)
    reverse_rtl_layer(fixture, relative)
    source = fixture / relative
    pre, post = keyboard.IMAGES[relative]
    current = keyboard.digest(source.read_bytes())
    if current == pre:
        return
    if current != post:
        raise ValueError(f'unexpected keyboard fixture image: {relative}')
    subprocess.run(['git', 'apply', '--reverse', '--whitespace=nowarn',
                    f'--include={relative}', str(ROOT / keyboard.PATCH)],
                   cwd=fixture, check=True, capture_output=True)
    if keyboard.digest(source.read_bytes()) != pre:
        raise ValueError(f'incorrect reversed keyboard fixture image: {relative}')
