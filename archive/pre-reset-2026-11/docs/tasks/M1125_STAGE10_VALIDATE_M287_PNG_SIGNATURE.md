# M1125 — Stage 10 / validate the M287 screenshot signature

The M287 GUI evidence aggregator checked screenshot paths and SHA-256 digests,
but it accepted arbitrary bytes when the evidence state claimed `passed`. A
hashed text file with a `.png` name could therefore be retained as a successful
screenshot.

The aggregator now requires the PNG signature for each supplied screenshot.
The regression rehashes a plain-text `.png` artifact and confirms the M287
report is invalid. Synthetic positive fixtures now begin with the PNG
signature.

Focused validation passed:
`python3 -m unittest scripts.tests.test_aggregate_candidate_evidence.AggregateCandidateEvidenceTests.test_m287_rejects_non_png_screenshot_even_when_hash_matches`
(1 test). The full candidate pipeline and GUI evidence acceptance remain open.

This checks the PNG file signature; it does not decode pixels or prove that
the screenshot is visually correct.
