# Focus scope

`FocusScope(id, focused: Binding<bool>, child)` owns one GPUI `FocusHandle` in a retained RustUI scope. The stable, nonempty ID must be unique among sibling focus scopes. The handle is attached to a GPUI group surrounding the child, so native focus-in and focus-out events include descendant controls. The wrapper uses GPUI's focus ownership and tab order; RustUI does not keep a separate focus graph.

The binding is both a request and a report of the current native scope focus while the scope is rendered. A change from false to true focuses the container if neither it nor a descendant is already focused. A change from true to false blurs the window only while focus remains inside this scope. Repeated renders with the same value do not request focus again. Native focus entry or exit writes true or false to the component owner's binding only when the value differs. `.tab_stop(true)` includes the container itself in keyboard traversal; the default is false. `.label(...)` gives its native Group accessibility node a name.

Focusing the scope requests focus on its container, not a private focus handle inside an opaque child control. This API does not restore a predecessor after blur. When the scope is removed, its subscriptions and retained handle are released. Removal does not write false into the component owner's binding, so a previously true binding can remain true and will request focus again if the scope is recreated. Callers that remove a focused scope should clear their own binding if that behavior is unwanted.

## Native acceptance evidence

Milestone 52 received bounded native GUI acceptance in `target/headstart/milestone52-focus-scope-gui.json` using `verification/M52 M55 Review.app`. The reviewed binary SHA-256 was `05fabb23fc5780056a01adeca40eab671a2704a740ba428638aab531d027ab9a`.

Observed interactions: request A reported A=true/B=false; Tab and text entry reached A, and typing remained in A across a parent render. Requesting B then settling native focus moved the report to A=false/B=true and wrote B. Clearing A while B was focused preserved B text entry. Pointer focus into A updated its binding; clearing focused A set both bindings false and subsequent typing changed neither field. A child counter reset when A was removed and restored, while its bound text remained.

The shared Milestones 52–55 accepted test log reports 160 passed and 0 failed (`target/headstart/milestone52-55-tests-accepted.log`). The reuse report confirms the successful build reused all-fresh `gpui-component 0.7.3`, `gpui-pre 0.3.9`, and `gpui-pre-platform 0.3.9` artifacts (`target/headstart/milestone52-55-reuse-report.json`).

Limits remain: focus reporting is bound to a rendered scope, and removing a focused scope does not clear the caller's binding. The gallery explicitly clears it; the general focus-removal binding policy remains open. The report also observed a transient A=true/B=true state after requesting B, before the following native focus event settled it. Optional scope tab-stop behavior and VoiceOver/screen-reader acceptance are incomplete. Programmatic scope focus does not target a private editor handle inside an opaque child control. These checks do not establish full focus or accessibility acceptance.
