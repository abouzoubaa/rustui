# O7: Proposed tracked dialog opening for the pinned GPUI Components snapshot

**Status: proposal only.** `patches/proposed/gpui-component-tracked-dialog.patch`
is a 272-line review artifact generated from pristine file copies under
`target/headstart/dialog-keyed-proposal/`. It has not been applied to `vendor/`
or a candidate crate, compiled, tested, or promoted. RustUI's Dialog scaffold
remains unimplemented. Approval of this patch and an isolated validation run
would be separate decisions.

## Source identity and boundaries

- Package: Longbridge `gpui-component 0.7.3`, from `gpui-kit` revision
  `67265a36e260ffe03c5aaebdcd8d654dbf892eab`; registry checksum in
  `Cargo.lock`: `465a363235cd4062ef5f0953fc096fc764ed6f53dfbd9f409f669f93ea3b1ff9`.
  This package is distinct from a Zed-published GPUI release.
- Pristine preimages (SHA-256): `src/root.rs`
  `c438aabc60f18252420a40e773e0aa4447fe89b00e27033c0ad7e5980b9c2749`,
  `src/window_ext.rs`
  `697b8a6867d1f12085ac6c976568745f6ba4ca21f5a9727c7a74e30523ed77eb`,
  `src/lib.rs`
  `50d77481bd99c62fa711689a2608add5bc355f700bdb72ebced910af8e62fda7`.
- Proposal patch SHA-256:
  `1da71c0eb4f232c724398dfd30b8fc119564efb3eb30e2c5fb2e0aa50b9cce72`.
  The three proposed postimages are retained under
  `target/headstart/dialog-keyed-proposal/after/src/` for review; that directory
  is not an active dependency source.
- Proposed postimage SHA-256: `src/root.rs`
  `a4093836f2a4c593dca766a3886b9b0e249b9b585e318861695098b3105be937`,
  `src/window_ext.rs`
  `df0511745df196b3261fd4cbcf88429add1c4778aacbeb391755b37a55ee390a`,
  `src/lib.rs`
  `42714cdd43aedd93c5201185e1079516cdf7289ded67a5f4ebabbc1e173bb2dc`.
  A complete patched package tree hash awaits an approved isolated candidate.
- The original `vendor/gpui-component-0.7.3/.cargo-checksum.json` SHA-256 is
  `d5b533efba197cc15bcb6756b935a786e846463c1ee431f876e8c334a7e638c6`.
  Its `LICENSE-APACHE` SHA-256 is
  `d1b0449e5478c574ba4f686c2656df7fe77d66821a61f8b6ed3378a58ed9a811`.
  Both must stay pristine. A patched candidate would use an isolated path source
  with its own postimage hash, never a rewritten registry checksum.

## Proposed upstream API

The existing `WindowExt::open_dialog`, `close_dialog`, and `close_all_dialogs`
retain their signatures and behavior. A separate `TrackedDialogExt` trait on
`Window` adds:

```rust
fn open_dialog_tracked<F>(&mut self, cx: &mut App, build: F) -> DialogId
where F: Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static;
fn has_dialog_id(&mut self, id: DialogId, cx: &mut App) -> bool;
fn close_dialog_by_id(&mut self, id: DialogId, cx: &mut App) -> bool;
```

`DialogId` is opaque, copyable, and allocated once per opening by a checked
process-wide counter. IDs do not collide across windows and are never reused.
The new trait avoids adding required methods to `WindowExt`, preserving external
implementations. The existing untracked opening also receives an internal ID;
callers that ignore it behave as before. The builder remains a reusable render
closure. It must capture a persistent child entity or state, not a consumed
`AnyElement` or a newly created entity on each rebuild.

`close_dialog_by_id` returns `false` without changing the stack for a stale ID
or an ID opened in another window. Closing the top opening restores its
recorded predecessor focus, clears focused-input registration and text selection,
and removes its builder. Closing a lower opening leaves the top dialog and its
focus/selection active; the next higher opening inherits the removed opening's
predecessor focus handle, so later top-down closes skip the removed shell. Both
paths notify the window root, which renumbers native dialog layers. Programmatic
close has the existing `close_dialog` callback semantics: it does not invoke the
native `on_close` callback. Native Escape, backdrop, close button, and Confirm
continue through the existing top-of-stack path; the removed ID then becomes
stale.

The patch also increments a checked stack generation on opens and closes.
Delayed focus restoration from a Confirm close uses that generation instead of
stack length: a remove/open sequence that returns to the same length cannot
restore focus to a released dialog. Closing all dialogs invalidates pending
restoration. These focus details need executable validation before adoption.

## RustUI use after an approved upstream candidate

A controlled `Dialog` can retain one `ComponentHost<C>` for arbitrary RustUI
content, pass its entity into each native builder invocation, and keep the
returned `DialogId` with the exact owner and window handle. Binding false or
owner removal would call `close_dialog_by_id`, even if another dialog is above
it. Native dismissals would update the owner's binding; stale owner actions
would remain weak. Localization should resolve labels from the active catalog
on rebuild, and child lifecycle must be checked on close and restore. This
RustUI adapter is outside the proposed upstream patch; a copied element cannot
stand in for a persistent content entity.

## Required validation before adoption

1. Intake an isolated copy of the pinned package and verify the registry,
   preimage, license, and patch hashes above. Apply the exact patch with no
   fuzz only to that candidate copy; retain pristine and postimage hashes.
2. Compile the pinned upstream crate and unchanged `WindowExt` call sites.
   Confirm `TrackedDialogExt` and `DialogId` are publicly usable. No such build
   or patch application has run for this proposal.
3. In native/root tests, open A, B, C. Close B by ID while C is top: C stays
   visible and focused, B's builder and child entity release, C closes to A,
   and A closes to the background focus. Repeat for bottom and top removal,
   stale and cross-window IDs, and repeated closes.
4. Exercise deferred Confirm close while another opening is removed or opened.
   Include a remove/open sequence that restores the original stack length;
   delayed focus must not jump to an obsolete target or erase newer pending
   restoration. Check native Escape, backdrop, close button, Confirm/Cancel,
   stacked layer ordering, selection scope, and callback counts.
5. In RustUI's later adapter, check owner-controlled open/close, nested owners,
   arbitrary retained child content/actions, locale updates, child mount/unmount,
   stale actions, and native accessibility/focus. Record source tests and native
   GUI observations separately. Any failed case keeps the candidate isolated.

The fallback is the current pinned upstream package with Dialog still planned.
If upstream later provides an equivalent keyed close contract, retire this
proposal in favor of that API after a separately approved revision change.
