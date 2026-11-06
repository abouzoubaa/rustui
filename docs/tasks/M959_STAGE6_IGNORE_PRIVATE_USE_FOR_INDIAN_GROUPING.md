# M959 — Stage 6 / ignore private-use subtags for Indian grouping

The locale-aware number formatter now checks the base locale after removing
extensions before applying English India grouping. This keeps `en-IN` on
Indian grouping while `en-x-in` uses standard three-digit grouping. The focused
integer/decimal regression passed 1/1. Full CLDR and broader locale/RTL
acceptance remain open.
