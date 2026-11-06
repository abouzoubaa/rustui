# M953 — Stage 8 / bound interior splitter by right-side capacity

`divider_effective_range` previously computed an interior splitter's maximum
from every sibling's minimum. That could include capacity in panels before the
splitter, although pinned GPUI `ResizableState::resize_panel_at_handle` expands
the selected panel by shrinking only panels to its right. The reported
`aria-valuemax` and keyboard clamp could therefore exceed the size the native
resize path can reach.

The maximum now holds preceding measured panel sizes fixed and subtracts only
the configured minima of panels to the right from the container extent. A
three-panel regression covers the case where earlier-panel capacity used to
inflate the interior splitter's maximum.

This is source-level coverage only. VoiceOver announcements, native resizing
after the exposed bound, and wider platform acceptance remain open; M586 does
not establish those cases.
