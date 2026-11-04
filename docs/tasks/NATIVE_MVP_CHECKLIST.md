# Proposed macOS developer-preview MVP checks

This is a provisional acceptance checklist for the macOS developer preview
discussed with the user. It does not replace the full production roadmap or
assert that the user has approved a publication strategy. Count a check only
when all of its stated evidence exists. The checks differ in size, so their
unweighted count is not a measure of code written or time remaining.

**Current count: 2/10 checks complete (20%).**

1. **Complete — separate local consumer:** an app outside the checkout builds
   using the public API and responds to native button and bound text input.
   Evidence: R03/R04 and R03/R06 entries in the remaining-work ledger.
2. **Complete — repeatable local setup:** a documented generator creates that
   workspace without replacing an existing directory; its fixture and an
   offline build from a path with spaces passed. Evidence: R03/R04 local
   consumer setup in the ledger and `examples/external-consumer/README.md`.
3. **Open — window and lifecycle:** current-candidate native launch, close,
   error handling, owner teardown and cancellation are verified end to end.
   Bounded native probes confirm root unmount, cancellation of one pending
   timer-backed owner task, and dispatch of an injected window-open error.
   Actual platform failure and broader cancellation cases still need evidence.
4. **Open — basic controls and input:** the supported MVP controls, bindings,
   disabled states and ordinary keyboard/text editing pass a documented
   current-candidate native workflow.
5. **Open — navigation and overlays:** the chosen navigation and modal paths
   pass the native workflow, including recovery after focus loss. In
   particular, an unfocused Components Sheet or Dialog cannot yet regain
   keyboard focus through Tab.
6. **Open — theme and ordinary motion:** the supported light/dark theme and
   animation paths are verified on the current candidate.
7. **Open — current native CI:** the chosen source and compiled macOS checks
   pass on the exact candidate in the trusted CI environment.
8. **Open — distribution workflow:** the user-selected way to obtain RustUI is
   documented and verified independently of the development checkout. The
   local source workflow above does not satisfy an installed SDK requirement.
9. **Open — dependency notices and review:** the five native missing notice
   files and license sufficiency are resolved for the candidate.
10. **Open — supported macOS release:** the supported OS/version scope and
    package workflow are decided and verified; signing and notarization are
    verified if required for the chosen distribution route.

Localization, Web/Wasm, accessibility and VoiceOver remain deferred by the
user. They are not counted as passed and remain required wherever the final
product scope calls for them. The production-readiness objective remains open.
