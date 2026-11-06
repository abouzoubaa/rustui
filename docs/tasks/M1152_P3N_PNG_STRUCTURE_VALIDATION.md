# M1152 — Validate paired GUI screenshot PNG structure

## Gap

P3N treated the eight-byte PNG signature as sufficient evidence that an M287
screenshot was a PNG image. A signature-only file with a matching SHA-256 was
accepted, and the aggregate report marked the GUI gate passed.

## Change

The M287 aggregator now checks PNG chunk framing and CRCs, the required first
IHDR with basic header values, at least one IDAT chunk, and a terminal empty
IEND chunk. The synthetic fixture now contains a structurally valid minimal
PNG, and a focused regression rejects a signature-only artifact.

This check establishes PNG structure and artifact identity only. It does not
decode pixels or establish visual correctness; those remain review inputs.

## Evidence

- Pre-fix reproduction: an eight-byte signature-only artifact with a matching
  digest returned `status=consistent` and `gate_states.gui=passed`.
- Focused M287 regressions and the full aggregation module: **59/59 passed**.
- `py_compile` passed for the aggregator and focused test module.
