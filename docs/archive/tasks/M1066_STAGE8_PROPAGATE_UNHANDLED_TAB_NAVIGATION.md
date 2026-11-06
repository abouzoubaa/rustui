# M1066 — Stage 8 / propagate TabView navigation with no enabled target

TabView now stops Previous/Next/Home/End key propagation only after it finds an
enabled tab to focus. When every tab is disabled, there is no focus target and
the key event remains available to ancestor handlers. The focused no-target
regression passed 1/1; scoped formatting and whitespace checks passed. This is
source-level coverage; native keyboard and accessibility acceptance remain
open.
