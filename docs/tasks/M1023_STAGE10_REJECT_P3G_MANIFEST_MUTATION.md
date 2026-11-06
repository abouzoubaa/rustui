# M1023 — Stage 10: reject P3G manifest mutation during reads

P3G now compares the manifest descriptor's device, inode, size, modification
time, and change time before and after its bounded read. A manifest changed
during the read is rejected instead of being parsed as stable candidate
evidence. The focused mutation regression passed 1/1.

This hardens snapshot intake only. No candidate was run; candidate
build/correctness, GUI/performance evidence, review, approval, and promotion
remain open.
