# L2: Searchable RustUI Lab story browser

## Implementation

The ten-button horizontal story selector is replaced by RustUI's existing
searchable, fixed-height `List` in `apps/rustui-lab/src/main.rs`. Its items come
from `component_catalog::STORIES` in registry order. `story_selection` is the
list's `Binding<Option<usize>>`; the selected row index resolves through the
same `STORIES` array to the `StoryId` used by the existing canvas and definition
lookup. The list is 220 points tall, retains its own scroll/search behavior, and
offers a query placeholder and a hint to clear the query when no rows match.

All existing canvas arguments, component bindings, action handlers, and demo
controls remain wired as before. The separate event-log entries for selecting
each named story were removed with the ten selector handlers; demo actions
continue to update the event log. Search and row-selection focus/interaction
still need native GUI review.

## Manual follow-up

- At a small window size, reach all ten stories by scrolling; verify the list
  does not force a wide horizontal layout.
- Search by full and partial story title, including no-match queries; confirm
  clearing the query restores all stories and the helper text explains recovery.
- Select each result and confirm its title, summary, canvas, state, and all
  existing demo controls still correspond to that story.
- Verify keyboard focus, arrow-key selection, activation, and retained search
  text after changing stories.

No tests, Cargo commands, or GUI acceptance were run for this change. Treat the
above interaction and visual checks as pending until observed in the native Lab.
