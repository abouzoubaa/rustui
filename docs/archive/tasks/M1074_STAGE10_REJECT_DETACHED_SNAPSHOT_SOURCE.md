# M1074 — Stage 10 / reject detached snapshot source directories

Snapshot validation now reopens each recorded source path beneath the pinned
candidate root after inventory and compares its device/inode identity with the
directory that was inventoried. This detects a moved source ancestor that
would leave the pinned leaf descriptor unchanged. A deterministic move-during-
scan regression passed 1/1. No candidate was executed.
