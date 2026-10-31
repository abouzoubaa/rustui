# KeyedList accessible row names

`KeyedListItem::new` now rejects empty and whitespace-only labels. Each label is
used as the accessible name of its virtualized row, so a blank label would
produce a row with no useful name. Nonblank labels are preserved verbatim,
including surrounding whitespace.

The focused `rustui` unit test covers empty and whitespace-only labels and
verbatim preservation of a nonblank label. This is source-level validation;
native accessibility-tree and screen-reader acceptance remain part of the
broader collection acceptance work documented in `R3B_KEYED_LIST.md`.
