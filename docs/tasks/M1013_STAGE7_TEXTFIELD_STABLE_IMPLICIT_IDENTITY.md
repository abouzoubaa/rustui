# M1013 — Stage 7: keep TextField identity stable across placeholder changes

TextField's implicit identity previously included its placeholder, so localizing or otherwise changing that label remounted the retained field and could discard its editing state. The implicit identity now uses the field kind under the existing owner/sibling slot; explicit IDs still take precedence.

The focused regression verifies identity across a placeholder change and separation between sibling fields. `./scripts/cargo-headstart test -p rustui --lib implicit_identity_survives_placeholder_localization_and_separates_siblings` passed.

Native focus, IME, locale-switching, and accessibility acceptance remain open.
