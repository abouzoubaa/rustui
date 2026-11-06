# M1248 — Stage 11: reject module-script type prefixes

The Web packaging preflight accepts exact `module` values and rejects longer
prefixes such as `modulex`. The focused packaging regression passed and
confirmed the previous bundle remains intact on rejection. This is packaging
source evidence; browser acceptance is tracked separately.
