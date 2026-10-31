# M902 — Stage 10 / protect candidate evidence across destination races

`run_candidate_pipeline.py` checks whether a candidate destination exists
before creating it. Another process can create the same destination between
that check and `mkdir`. If the losing invocation then tries to write
`pipeline-failure.json`, it can corrupt the winning run's evidence.

The runner now tracks whether its own `mkdir` completed. The exception handler
records failure evidence only for a destination created by that invocation. A
focused simulated race makes a competing invocation create the destination
immediately before the runner's create call fails, and confirms the winner's
directory does not acquire failure evidence from the loser.

Focused check passed:

```sh
python3 -m unittest scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_destination_race_does_not_write_into_other_run
```

This protects local evidence isolation. It does not validate a candidate's
provenance, build, correctness, native GUI or performance behavior, and does
not review, approve, or promote a candidate.
