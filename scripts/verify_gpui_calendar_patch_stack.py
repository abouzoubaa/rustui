#!/usr/bin/env python3
"""Verify the calendar keyboard layer and its exact earlier patch images."""
import importlib.util
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('calendar_keyboard', ROOT / 'scripts/apply_gpui_calendar_keyboard_patch.py')
keyboard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(keyboard)


def main():
    # The keyboard verifier checks both live files, patch identity and reversal.
    keyboard.run(ROOT, ROOT, '--verify')
    with tempfile.TemporaryDirectory(prefix='rustui-calendar-stack-') as folder:
        staged = Path(folder)
        files = [Path('Cargo.toml'), *keyboard.IMAGES, *map(Path, keyboard.PRESERVED)]
        files += [Path(f'vendor/{name}-0.7.3/Cargo.toml') for name in ('gpui-base', 'gpui-component')]
        files += [Path('vendor/gpui-base-0.7.3/LICENSE-APACHE'),
                  Path('patches/approved/gpui-component-date-picker-localization-v2.patch')]
        for relative in files:
            target = staged / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        keyboard.run(staged, ROOT, '--reverse')
        commands = [
            ('apply_gpui_base_calendar_date_limits.py', '--verify', True),
            ('apply_gpui_date_picker_rtl_patch.py', '--verify', True),
            ('apply_gpui_component_date_picker_patch.py', '--check', False),
        ]
        for script, mode, supports_artifacts in commands:
            command = [sys.executable, str(ROOT / 'scripts' / script), '--root', str(staged)]
            if supports_artifacts:
                command += ['--artifacts', str(ROOT)]
            subprocess.run(command + [mode], check=True)
    print('calendar patch stack verified')


if __name__ == '__main__':
    main()
