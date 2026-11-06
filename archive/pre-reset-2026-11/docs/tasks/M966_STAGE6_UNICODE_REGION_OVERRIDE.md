# M966 — Unicode region override for number grouping

Stage 6 / locale-aware formatting.

For English locales, the formatter now checks a valid Unicode `u-rg-…zzzz`
region override before falling back to the base locale region. `en-US-u-rg-inzzzz`
uses Indian grouping; `en-IN-u-rg-uszzzz` uses western grouping.

The focused integer/decimal grouping regression passed 1/1. This adds one
region-override rule to the existing bounded formatter; full CLDR formatting
remains open.
