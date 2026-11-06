# L2: Searchable RustUI Playground catalog and framework demos

## Implementation

The canonical app is `apps/rustui-playground`. Its searchable, scrollable
component catalog is separate from the ten framework demos. The catalog and
demo definitions are registered in `apps/rustui-playground/src/component_catalog.rs`;
the shell, query, selection, and demo switching are wired in
`apps/rustui-playground/src/lib.rs`. Demo implementations live under
`apps/rustui-playground/src/stories/`.

Selecting a component catalog row inspects its packaged SVG references.
Selecting a framework demo opens its RustUI example and the corresponding
illustrative source fragment. The source display is not compiled or evaluated
by the Playground. `rustui-lab` is a compatibility launcher and does not own a
second catalog or story implementation.

The current native observation record is
`target/headstart/evidence/playground-l1-native-20261021.json`. Search, no-match
recovery, catalog scrolling, and keyboard interaction still need dedicated
acceptance; seeing ten demos in the UI does not verify those behaviors. The
sidebar now shows `No matches. Clear the search to show all components and demos.`
for a nonempty query with no catalog or story results. Its result check uses
localized catalog coverage labels, so a translated coverage match does not
trigger the empty-state message.

## Manual follow-up

- Search by full and partial component/demo title, including no-match queries;
  confirm clearing the query restores results and helper text explains
  recovery.
- At a small window size, reach every catalog item and all ten framework demos
  by scrolling; verify the layout does not force a wide horizontal selector.
- Select each framework demo and confirm its title, summary, rendered content,
  state, event summary, and source fragment correspond.
- Verify keyboard focus, arrow-key selection, activation, and retained search
  text after changing demos.
- Switch repeatedly among demos and ensure catalog selection, query state, and
  demo state remain coherent.

Record results from the native Playground window. Compilation and source
inspection do not establish search, scrolling, focus, or selection behavior.
