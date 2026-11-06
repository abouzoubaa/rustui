# M1126 — Stage 7 / keep a controlled DatePicker year selectable

The pinned calendar initially exposes a year range around the current year.
RustUI accepts any valid controlled `NaiveDate`, so a selected date outside
that range could be displayed while its year was unavailable in the year
selector.

DatePicker now sets its calendar range around the selected year both when
creating retained state and when syncing a changed controlled value. The
focused regression verifies that a 1900 selection is within the resulting
range. The existing bounds use a half-open range.

Focused validation passed:
`./scripts/cargo-headstart test -p rustui selected_date_year_is_in_the_calendar_year_range`
(1 passed, 845 filtered).

Native calendar interaction and accessibility acceptance remain open.
