# M1070 — Stage 9 / avoid duplicate property editor labels

The property inspector no longer adds an external label for Toggle and
Integer/Stepper editors, which already render their own visible labels. Text,
Choice, and integer-value editor rows continue to receive the external label.
The focused duplicate-label regression passed 1/1. This is a bounded
Playground inspector correction; broader UI Studio composition and acceptance
remain open.
