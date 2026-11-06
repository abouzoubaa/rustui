# F1a Keyed Lab Demonstration

The existing Layout story demonstrates keyed collection behavior without adding a new catalog entry. Two `TextEditor` rows use stable semantic keys (`'alpha'` and `'beta'`) and the same local editor identity (`"row-editor"`). The collection is eagerly built in either order, and the Beta row is represented as `Option<KeyedView<_>>` so the same child list demonstrates conditional omission and restoration.

The row text is owned by bindings on the Lab root, so it survives removal of Beta. Cursor and undo state stay with each keyed editor during reorder. Removing Beta prunes its native editor; restoring it creates a new editor, so its local cursor and undo state reset. Keyboard focus restoration has not been verified. The live Lab and GUI behavior have not been run as part of this F1a implementation.
