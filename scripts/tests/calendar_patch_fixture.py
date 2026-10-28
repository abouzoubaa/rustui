"""Normalize one exact keyboard-patched source in an isolated fixture."""
import importlib.util
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('calendar_keyboard_fixture', ROOT / 'scripts/apply_gpui_calendar_keyboard_patch.py')
keyboard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(keyboard)


def reverse_keyboard_layer(fixture, relative):
    relative = Path(relative)
    source = fixture / relative
    pre, post = keyboard.IMAGES[relative]
    current = keyboard.digest(source.read_bytes())
    if current == pre:
        return
    if current != post:
        raise ValueError(f'unexpected keyboard fixture image: {relative}')
    keyboard.run(ROOT, ROOT, '--verify')
    subprocess.run(['git', 'apply', '--reverse', '--whitespace=nowarn',
                    f'--include={relative}', str(ROOT / keyboard.PATCH)],
                   cwd=fixture, check=True, capture_output=True)
    if keyboard.digest(source.read_bytes()) != pre:
        raise ValueError(f'incorrect reversed keyboard fixture image: {relative}')
