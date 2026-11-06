# M809 — Stage 7 DatePicker minimum month count

The DatePicker builder clamps `number_of_months(0)` to one. Pinned
`gpui-base::Calendar::number_of_months` has the same minimum-one behavior, so
this test protects the RustUI boundary from passing through an invalid count.

The focused regression `zero_month_count_is_clamped_to_one` passed, along with
a scoped diff check. Native calendar interaction and accessibility acceptance
remain open.
