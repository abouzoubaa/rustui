# M960 — Stage 8 / skip disabled Sidebar boundary selections

Up/Down navigation from a disabled current row at either list boundary now
falls back to an enabled row rather than retaining the disabled row when an
enabled row exists. The focused regression passed 1/1. Native keyboard-routing
and accessibility acceptance remain open.
