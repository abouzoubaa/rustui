# M1235 — Reject P3N PNGs without image-data bytes

P3N structural validation no longer accepts a CRC-valid PNG containing an
empty `IDAT` chunk followed by `IEND`. The validator now requires at least one
byte across the PNG image-data chunks. The focused M287 aggregation regression
passed, and the validator and focused test module passed `py_compile`.

This checks PNG structure only; it does not decode pixels or establish visual
or GUI acceptance. Current-source capture, image review, performance evidence,
independent review, approval, and promotion remain open.
