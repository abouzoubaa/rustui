# M949 — Stage 3 / reconcile the Search Field catalog expectation

The catalog's `search_field_examples` note documents its expanded shared
Text-input composition and native acceptance gap. The aggregate catalog test
still required the generic baseline note, so it failed even though its
dedicated Search Field contract test passed. The aggregate test now checks the
documented Search Field coverage instead of the obsolete baseline text.

Focused validation: `./scripts/cargo-headstart test -p rustui-playground --lib catalog::tests::search_matches_name_api_and_status --locked --offline` passed (1/1).
