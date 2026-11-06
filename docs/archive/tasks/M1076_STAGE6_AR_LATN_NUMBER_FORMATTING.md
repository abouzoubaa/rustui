# M1076 — Stage 6 / honor the explicit Latn script in Arabic number formatting

Locale formatting now uses Latin digits, comma grouping, and a dot decimal for
`ar-Latn`, while retaining the Arabic locale's left-to-right mark on signed
numbers. The separate `ar-EG-u-nu-latn` numbering-system override keeps its
existing Arabic separators. The focused integer/decimal regression passed
1/1.
