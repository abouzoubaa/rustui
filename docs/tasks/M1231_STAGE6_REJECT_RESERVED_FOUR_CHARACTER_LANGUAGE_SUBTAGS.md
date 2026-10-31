# M1231 — Superseded locale-parser correction

The four-letter primary-language rejection originally recorded here was
incorrect and has been reverted. RFC 5646 §2.1 permits `4ALPHA` as a language
subtag reserved for future use. See the correction and passing positive
regression in
`docs/tasks/M1239_STAGE6_PRESERVE_RESERVED_FOUR_CHARACTER_LANGUAGE_SUBTAGS.md`.
