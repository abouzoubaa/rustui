# M1214 — Stage 10 require distinct P3D screenshots

P3D validation now rejects a captured state whose screenshot artifact shares
file identity with the build record, plan, or candidate binary. A provenance
artifact could previously be referenced as a screenshot while passing
structural validation, despite P3D requiring separate captured images.

Verification: `python3 scripts/tests/test_validate_p3d_visual_evidence.py
ScreenshotDistinctnessTests.test_rejects_screenshot_for_not_run_state` passed
(1 test). `py_compile` for the validator succeeded. No candidate was run and
no visual acceptance is claimed.
