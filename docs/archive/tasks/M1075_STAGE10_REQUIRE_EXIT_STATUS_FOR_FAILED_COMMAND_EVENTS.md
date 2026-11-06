# M1075 — Stage 10 / require exit status for failed command evidence

Evidence aggregation now requires a failed command-backed pipeline event to
contain a nonzero integer `exit_status`, matching the pipeline runner's
recorded process results. Commandless internal failures retain their existing
representation. The new invalid-record regression and the valid failed-pipeline
case passed (2 tests); Python compilation and scoped whitespace checks passed.
No candidate was executed.
