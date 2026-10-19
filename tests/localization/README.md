# localization verification

Planned scope: Messages, pluralization, formatting, RTL, bidi, fallback, and runtime switching.

No tests are implemented in this directory. Root workspace `tests/` directories
are not automatically discovered by Cargo. Place executable integration tests in
the owning package's `tests/` directory or declare an explicit future harness.
Visual evidence and native platform checks are separate from compilation.
