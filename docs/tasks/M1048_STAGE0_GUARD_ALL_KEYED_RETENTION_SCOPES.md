# M1048 — Guard every keyed retention scope against duplicate siblings

`RetainedEntities::push_scope` checked duplicate sibling scope identities for most keyed scopes, but omitted `Tabs`, `Tab`, `NavigationSplit`, and `Page`. Repeating one of these scopes under the same parent could let two siblings reuse the same retained native identity.

The duplicate-scope guard now includes all four variants. One focused regression exercises each omitted scope.

Focused check:

```text
./scripts/cargo-headstart test -p rustui --lib runtime::component_host::context_menu_scope_tests::remaining_keyed_retention_scopes_reject_duplicate_siblings -- --exact
```

Result: passed (1 test). Native navigation, tab interaction, and lifecycle acceptance remain open.
