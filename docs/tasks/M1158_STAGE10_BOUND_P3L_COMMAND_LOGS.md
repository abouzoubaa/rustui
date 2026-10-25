# M1158 — Stage 10 bound P3L command logs

P3L previously loaded hash-bound command logs into memory without a size
limit. A self-consistent 2 MiB event log was accepted as `consistent`, and
larger candidate-controlled logs could consume unbounded memory during local
aggregation.

Command event logs now have a 64 MiB per-file limit. The P3F validator output
log retains its tighter 1 MiB record limit. Oversized command logs invalidate
the aggregate report; no gate is promoted and no candidate command is run.

Focused checks passed 4/4:

- `test_command_event_log_size_is_bounded`
- `test_p3f_validator_output_record_size_is_bounded`
- `test_failed_command_event_requires_hash_bound_log`
- `test_pipeline_event_record_size_is_bounded`

This bounds P3L command-log reads. It does not establish candidate acceptance,
GUI/performance acceptance, or promotion.
