#!/usr/bin/env python3
"""Record a bounded W1 session; a caller-created stop file closes the window.

The target is never launched or signalled. Reuses the process collector's units,
PID identity guard, partial evidence handling, and exclusive JSON writer.
"""
import argparse
import hashlib
from pathlib import Path

from collect_runtime_metrics import collect, positive_ns, run_id_arg
from runtime_metrics import default_output_path, write_json_exclusive


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pid', type=int, required=True)
    parser.add_argument('--run-id', type=run_id_arg, required=True)
    parser.add_argument('--stop-file', type=Path, required=True)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--interval', type=positive_ns, default=positive_ns('0.25'))
    parser.add_argument('--max-duration', type=positive_ns, default=positive_ns('600'))
    args = parser.parse_args()
    if args.pid <= 0 or args.interval > args.max_duration:
        parser.error('positive PID and interval <= maximum duration required')
    if args.stop_file.exists() or args.stop_file.is_symlink():
        parser.error('stop file must not already exist')
    if not args.binary.is_file():
        parser.error('binary must be an existing file')
    identity = {'path': str(args.binary.resolve()),
                'sha256': hashlib.sha256(args.binary.read_bytes()).hexdigest(),
                'binding': 'caller-supplied binary; process path independently recorded'}
    document = collect(args.pid, args.max_duration, args.interval,
                       args.run_id, 'W1-30-cycles-native-AX', args.run_id,
                       stop_requested=args.stop_file.exists)
    document['binary_reference'] = identity
    document['termination'] = ('stop_file' if args.stop_file.exists() else 'maximum_duration')
    if document['termination'] != 'stop_file' and document['completion']['status'] == 'complete':
        document['completion'] = {'status': 'incomplete', 'error': 'W1 stop file was not received'}
    destination = default_output_path(args.run_id, 'process')
    write_json_exclusive(destination, document)
    print(destination)
    return 0 if document['completion']['status'] == 'complete' else 1


if __name__ == '__main__':
    raise SystemExit(main())
