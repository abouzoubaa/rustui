# M1241 — Bound approved patch copies

The Stage 10 candidate pipeline rejects approved patch inputs larger than
32 MiB before creating their destination copies. The focused regression
`test_patch_copy_rejects_oversized_input_before_creating_destination` passed
(1/1).

This is a bounded intake-copy safeguard. It does not prove candidate execution,
candidate correctness, GUI or performance acceptance, independent review,
approval, or promotion.
