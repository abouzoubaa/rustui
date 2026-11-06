# L3c: RustUI Lab scroll host

## Scope

The Lab's existing root content is hosted in a vertical native GPUI viewport
with identity `lab-content-scroll`. It fills the available size by default and
keeps the existing `VStack` spacing and start alignment. This makes content
below the initial window height reachable, including the story canvas,
contextual controls, and source and event panels.

The story list retains its fixed height, search filter, and nested scrolling.
Lab theme and locale selection, bindings, controls, and state remain unchanged.

## Acceptance status

Expected behavior is native vertical scrolling of the Lab content, with the
inner story list continuing to scroll and filter independently. This behavior
has not been verified in a running GUI. Nested-scroll gesture handoff and
keyboard/accessibility navigation still require native GUI acceptance.

## Combined integration evidence

The final locked all-target workspace check passed with exit 0 (1.87 seconds),
recorded in `target/headstart/roadmap-milestone6-check-final.{jsonl,log}`.
Non-recursive rustfmt checking passed. Required GPUI dependency check artifacts
were all fresh. No tests or GUI/accessibility checks ran; native scrolling remains
an implemented contract awaiting interactive acceptance.
