# M1232 — Close P3K package descriptors on setup failure

P3K now keeps candidate-root duplication and patch-check destination setup
inside the cleanup scope. The package-directory descriptors are closed on
preflight rejection, setup failure, and later patch-check exits. The focused
regression forces patch-check setup to fail after preflight and confirms both
package descriptors are closed; it passed (1/1).

This closes a patch-applier resource leak. Candidate execution, downstream
build/correctness, GUI/performance evidence, independent review, approval, and
promotion remain separate gates.
