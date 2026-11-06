# Collection accessibility

`Collection` exposes a named `List` root and named `ListItem` rows. Each row carries one-based position, set size, and selected state through GPUI's AccessKit element metadata.

## M554 serialization regression

List builder emits Role::List and its accessible name. Row builder emits Role::ListItem, its name, one-based position, set size, and selected state. Serialization suite passed 9/9 in `target/headstart/evidence/m554-collection-accesskit-serialization-20261021.log`. Full-window tree and screen-reader acceptance remain open.

This checks element serialization. Full-window tree relationships, screen-reader announcements, keyboard behavior, RTL, and cross-platform acceptance remain separate checks.
