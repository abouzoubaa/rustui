# M1091 — Stage 4: keep indexed constructor postfixes in stack children

The stack DSL parser now keeps an index suffix attached to its constructor
expression before parsing the next sibling. Indexed constructors are
recognized recursively, and their outer attributes remain attached so cfg
gating still applies.

The focused regression passed 1/1. This is a parser-boundary check; broader
macro, Playground, and platform acceptance remain separate.
