# Picker owner-selection reconciliation

Picker now reconciles its retained Select state only when its option list or
controlled owner selection changes. An unchanged owner value can be absent
from the options, so comparing the retained selected value to the owner value
on every render repeatedly calls `set_selected_value`. For searchable Picker
instances, that resets the active query and can also discard an unconfirmed
retained selection during an unrelated owner rerender.

The retained owner snapshot distinguishes an actual external selection change
from a local in-progress Select interaction. Option changes still reconcile
once, and real owner changes still take precedence. The regression covers an
unchanged unmatched owner value, an external value change, and an option-list
change.

This is source-level behavior coverage; native searchable-Picker interaction,
keyboard/focus behavior, VoiceOver, and cross-platform acceptance remain open.
