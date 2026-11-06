# M1060 — Isolate candidate pipeline pinned-FD context per invocation

## Problem

The candidate pipeline stored its active pinned-directory descriptors and child
environment in a module global. Concurrent `run_pipeline` calls in one Python
process could overwrite that value while another call was preparing or running a
validator. The second call could then make file operations or child processes use
the first call's repository and candidate descriptors.

## Change

Store the active context in a `ContextVar`. Each invocation now sees and restores
its own context, including nested invocations, and concurrent threads do not
cross-wire validator environments or inherited descriptors.

## Focused evidence

`python3 -m unittest scripts.tests.test_run_candidate_pipeline.CandidatePipelineTests.test_concurrent_run_steps_keep_pinned_fd_context_isolated`

Result: 1 passed. The regression synchronizes two concurrent validator launches
and checks each launch receives its own context marker and descriptor tuple.

No candidate was executed or approved, and no patch was promoted. No broader
tests or builds were run.
