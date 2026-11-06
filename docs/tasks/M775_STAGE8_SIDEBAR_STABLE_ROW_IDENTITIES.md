# M775 — Stage 8 Sidebar row identity across reordering

Sidebar menu rows now use a stable element identity derived from the Sidebar ID
and each item ID instead of their current row position. Reordering items no
longer transfers retained row identity to a different item. The GPUI Components
row renderer remains responsible for the row's selection, accessibility, and
interaction behavior.

The focused Sidebar identity regression, scoped formatting, and whitespace
checks passed. Native accessibility-tree, VoiceOver, and full keyboard
acceptance remain open.
