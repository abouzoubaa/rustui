#!/bin/sh
# Package and check one isolated Playground instance without touching other instances.
set -eu
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
. "$repo_root/scripts/pinned-toolchain.sh"
dry_run=0
case "$#" in
    0) ;;
    1) [ "$1" = "--dry-run" ] || { printf '%s\n' "error: unknown argument: $1" >&2; exit 2; }; dry_run=1 ;;
    *) printf '%s\n' 'error: expected no arguments or --dry-run' >&2; exit 2 ;;
esac
scratch=$(mktemp -d "${TMPDIR:-/tmp}/rustui-playground-smoke.XXXXXX") || { printf '%s\n' 'error: cannot create temporary directory' >&2; exit 1; }
scratch=$(CDPATH= cd -- "$scratch" && pwd -P)
bundle="$scratch/RustUI Playground.app"
executable="$bundle/Contents/MacOS/rustui-playground"
app_pid=
open_pid=
smoke_ok=0
process_matches() {
    python3 - "$app_pid" "$executable" <<'PY'
import subprocess, sys
r = subprocess.run(["ps", "-p", sys.argv[1], "-o", "command="], capture_output=True, text=True)
command = r.stdout.strip()
sys.exit(0 if r.returncode == 0 and (command == sys.argv[2] or command.startswith(sys.argv[2] + " ")) else 1)
PY
}
cleanup() {
    result=$?
    trap - EXIT
    if [ -n "$app_pid" ] && process_matches; then
        # NSRunningApplication sends a graceful quit request to this PID only.
        /usr/bin/osascript -l JavaScript - "$app_pid" >/dev/null 2>&1 <<'JXA' || true
ObjC.import('AppKit');
function run(argv) {
    const app = $.NSRunningApplication.runningApplicationWithProcessIdentifier(Number(argv[0]));
    app.terminate;
}
JXA
        for _ in 1 2 3 4 5; do
            process_matches || break
            sleep 1
        done
        if process_matches; then kill "$app_pid" 2>/dev/null || true; fi
        for _ in 1 2 3 4 5; do
            process_matches || break
            sleep 1
        done
        if process_matches; then
            kill -KILL "$app_pid" 2>/dev/null || true
            for _ in 1 2 3 4 5; do
                process_matches || break
                sleep 1
            done
        fi
        if process_matches; then
            printf '%s\n' 'error: temporary Playground process did not terminate' >&2
            result=1
        fi
    fi
    if [ -n "$open_pid" ]; then
        kill "$open_pid" 2>/dev/null || true
        wait "$open_pid" 2>/dev/null || true
    fi
    rm -rf -- "$scratch" || result=1
    if [ "$smoke_ok" -eq 1 ] && [ "$result" -eq 0 ]; then
        printf '%s\n' 'Playground smoke passed: temporary app stayed running for 10 seconds with no new crash report.'
    fi
    exit "$result"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
if [ "$dry_run" -eq 1 ]; then
    printf 'temporary bundle: %s\n' "$bundle"
    exit 0
fi
console_user=$(stat -f '%Su' /dev/console 2>/dev/null || true)
if [ "$console_user" != "$(id -un)" ] || ! launchctl print "gui/$(id -u)" >/dev/null 2>&1; then
    printf '%s\n' 'no GUI session' >&2
    exit 1
fi
cd "$repo_root"
if ! pinned_cargo build -p rustui-playground --locked --offline; then
    printf '%s\n' 'error: Playground build failed' >&2
    exit 1
fi
if ! RUSTUI_PLAYGROUND_BUNDLE_PATH="$bundle" apps/rustui-playground/package-app.sh --replace; then
    printf '%s\n' 'error: Playground packaging failed' >&2
    exit 1
fi
marker="$scratch/launched"
touch "$marker"
/usr/bin/open -g -n -W "$bundle" >"$scratch/open.log" 2>&1 &
open_pid=$!
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
    app_pid=$(python3 - "$executable" <<'PY'
import subprocess, sys
p = subprocess.run(["ps", "-axo", "pid=,command="], capture_output=True, text=True, check=True)
for line in p.stdout.splitlines():
    fields = line.strip().split(None, 1)
    if len(fields) == 2 and (fields[1] == sys.argv[1] or fields[1].startswith(sys.argv[1] + " ")):
        print(fields[0]); break
PY
)
    [ -n "$app_pid" ] && break
    sleep 1
done
if [ -z "$app_pid" ]; then
    printf '%s\n' 'error: Playground did not launch from the temporary bundle' >&2
    cat "$scratch/open.log" >&2
    exit 1
fi
sleep 10
if ! process_matches; then
    printf '%s\n' 'error: Playground exited during the 10-second smoke check' >&2
    exit 1
fi
if ! python3 - "$HOME/Library/Logs/DiagnosticReports" "$marker" "$app_pid" <<'PY'
from pathlib import Path
import re, sys
root, marker, pid = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
pattern = re.compile(r'(?:(?:"pid"\s*:\s*)|(?:Process:[^\n]*\[\s*))' + re.escape(pid) + r'\b')
if root.is_dir():
    for path in root.iterdir():
        if 'rustui' not in path.name.lower() or 'playground' not in path.name.lower() or path.suffix not in ('.ips', '.crash'):
            continue
        if path.stat().st_mtime_ns > marker.stat().st_mtime_ns and pattern.search(path.read_text(errors='replace')):
            print(f'error: new Playground crash report: {path}', file=sys.stderr)
            sys.exit(1)
PY
then
    exit 1
fi
smoke_ok=1
