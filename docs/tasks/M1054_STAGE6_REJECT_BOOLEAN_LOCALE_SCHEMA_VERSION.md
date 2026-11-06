# M1054 — Stage 6 reject Boolean locale schema versions

The locale metadata validator now requires `schema_version` to be an integer
before comparing it with version 1. Python treats `True == 1`, so the previous
comparison accepted a Boolean as a valid schema version.

A focused regression invokes the standalone locale validator with a Boolean
schema version and requires rejection. Native locale switching, rendered locale
output, RTL layout, and screen-reader acceptance remain separate open checks.
