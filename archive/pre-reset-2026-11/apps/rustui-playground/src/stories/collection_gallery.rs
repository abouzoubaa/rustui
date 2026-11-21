//! Virtualized Collection with stable IDs and caller-owned row data.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use rustui::prelude::*;

const INITIAL_ROWS: usize = 1_000;

pub fn demo() -> impl View {
    Component("collection-gallery", CollectionGallery::default)
}

#[derive(Clone, PartialEq, Eq)]
struct RowSnapshot {
    id: Identity,
    label: String,
    note: String,
    count: u32,
}

#[derive(Clone)]
struct RowRecord {
    label: String,
    note: String,
    count: u32,
}

#[derive(Default)]
struct LifecycleCounts {
    active: HashSet<Identity>,
    mounts: u64,
    releases: u64,
}

struct RowStore {
    records: HashMap<Identity, RowRecord>,
    lifecycle: LifecycleCounts,
}

impl Default for RowStore {
    fn default() -> Self {
        let records = (0..INITIAL_ROWS)
            .map(|index| {
                let id = Identity::from(format!("record-{index:04}"));
                let label = format!("Record {index:04}");
                (
                    id,
                    RowRecord {
                        note: format!("Note for record {index:04}"),
                        label,
                        count: 0,
                    },
                )
            })
            .collect();
        Self {
            records,
            lifecycle: LifecycleCounts::default(),
        }
    }
}

type SharedRows = Rc<RefCell<RowStore>>;

#[derive(Clone, PartialEq)]
struct RemovedRow {
    id: Identity,
    previous: Option<Identity>,
    next: Option<Identity>,
    index: usize,
    reversed: bool,
}

#[component]
struct CollectionGallery {
    #[state(initial_order())]
    order: Vec<Identity>,
    #[state(None)]
    selected: Option<Identity>,
    #[state(None)]
    removed: Option<RemovedRow>,
    #[state(false)]
    reversed: bool,
    #[state(0)]
    confirm_count: u32,
    #[state(0)]
    diagnostic_refresh: u32,
    rows: SharedRows,
}

fn initial_order() -> Vec<Identity> {
    (0..INITIAL_ROWS)
        .map(|index| Identity::from(format!("record-{index:04}")))
        .collect()
}

fn restored_index(order: &[Identity], removed: &RemovedRow, reversed: bool) -> usize {
    let (first_anchor, second_anchor) = if reversed == removed.reversed {
        (removed.next.as_ref(), removed.previous.as_ref())
    } else {
        (removed.previous.as_ref(), removed.next.as_ref())
    };

    if let Some(index) = first_anchor.and_then(|anchor| order.iter().position(|id| id == anchor)) {
        return index;
    }

    if let Some(index) = second_anchor.and_then(|anchor| order.iter().position(|id| id == anchor)) {
        return index + 1;
    }

    removed.index.min(order.len())
}

impl CollectionGallery {
    fn selection_binding(&self) -> Binding<Option<Identity>> {
        Binding::new(self.selected(), Self::read_selection, Self::write_selection)
    }

    fn read_selection(owner: &Self) -> Option<Identity> {
        owner.selected()
    }

    fn write_selection(owner: &mut Self, selection: Option<Identity>) {
        owner.selected = selection;
    }

    fn reverse_order(&mut self) {
        self.order.reverse();
        self.reversed = !self.reversed;
    }

    fn select_last(&mut self) {
        self.selected = self.order.last().cloned();
    }

    fn clear_selection(&mut self) {
        self.selected = None;
    }

    fn remove_selected(&mut self) {
        let Some(selected) = self.selected.clone() else {
            return;
        };
        let Some(index) = self.order.iter().position(|id| *id == selected) else {
            return;
        };
        let removed = RemovedRow {
            id: selected,
            previous: index
                .checked_sub(1)
                .and_then(|index| self.order.get(index))
                .cloned(),
            next: self.order.get(index + 1).cloned(),
            index,
            reversed: self.reversed,
        };
        self.order.remove(index);
        self.removed = Some(removed);
    }

    fn restore_removed(&mut self) {
        let Some(removed) = self.removed.take() else {
            return;
        };
        let index = restored_index(&self.order, &removed, self.reversed);
        self.order.insert(index, removed.id);
    }

    fn confirm_selected(&mut self) {
        if self.selected.is_some() {
            self.confirm_count += 1;
        }
    }

    fn refresh_diagnostics(&mut self) {
        self.diagnostic_refresh += 1;
    }
}

#[derive(Clone)]
struct RowProps {
    snapshot: RowSnapshot,
    store: SharedRows,
}

impl PartialEq for RowProps {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot == other.snapshot && Rc::ptr_eq(&self.store, &other.store)
    }
}

impl Eq for RowProps {}

#[component]
struct CollectionRow {
    id: Option<Identity>,
    label: String,
    store: Option<SharedRows>,
    #[state(String::new())]
    note: String,
    #[state(0)]
    count: u32,
}

impl CollectionRow {
    fn apply_props(&mut self, props: &RowProps) {
        self.id = Some(props.snapshot.id.clone());
        self.label.clone_from(&props.snapshot.label);
        self.store = Some(props.store.clone());
        if self.note() != props.snapshot.note {
            self.note.clone_from(&props.snapshot.note);
        }
        if self.count() != props.snapshot.count {
            self.count = props.snapshot.count;
        }
    }

    fn bound_note(&self) -> Binding<String> {
        Binding::new(self.note(), Self::read_note, Self::write_note)
    }

    fn read_note(owner: &Self) -> String {
        owner.note()
    }

    fn write_note(owner: &mut Self, value: String) {
        owner.note.clone_from(&value);
        if let (Some(id), Some(store)) = (&owner.id, &owner.store) {
            if let Some(record) = store.borrow_mut().records.get_mut(id) {
                record.note = value;
            }
        }
    }

    fn increment(&mut self) {
        self.count += 1;
        if let (Some(id), Some(store)) = (&self.id, &self.store) {
            if let Some(record) = store.borrow_mut().records.get_mut(id) {
                record.count = self.count;
            }
        }
    }
}

impl View for CollectionRow {
    fn body(&self) -> impl View {
        let id = self.id.as_ref().map_or("unknown", Identity::as_str);
        let action = self.action(Self::increment);

        VStack::new()
            .spacing(3.0)
            .child(Text(format!("{} · count {}", self.label, self.count())))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        TextField(format!("{} note", self.label), self.bound_note())
                            .id(format!("collection-note-{id}")),
                    )
                    .child(
                        Button(
                            format!("Increment {} · {}", self.label, self.count()),
                            action,
                        )
                        .id(format!("collection-increment-{id}"))
                        .accessibility_label(format!(
                            "Increment {} counter, currently {}",
                            self.label,
                            self.count()
                        )),
                    ),
            )
    }

    fn on_mount(&mut self, _: &mut rustui::App) {
        if let (Some(id), Some(store)) = (&self.id, &self.store) {
            let mut store = store.borrow_mut();
            store.lifecycle.mounts += 1;
            store.lifecycle.active.insert(id.clone());
        }
    }

    fn on_unmount(&mut self, _: &mut rustui::App) {
        if let (Some(id), Some(store)) = (&self.id, &self.store) {
            let mut store = store.borrow_mut();
            store.lifecycle.releases += 1;
            store.lifecycle.active.remove(id);
        }
    }
}

impl View for CollectionGallery {
    fn body(&self) -> impl View {
        let order = self.order();
        let rows = self.rows.clone();
        let items = {
            let store = rows.borrow();
            order
                .iter()
                .filter_map(|id| {
                    let record = store.records.get(id)?;
                    Some(RowSnapshot {
                        id: id.clone(),
                        label: record.label.clone(),
                        note: record.note.clone(),
                        count: record.count,
                    })
                })
                .map(|snapshot| {
                    CollectionItem::new(snapshot.id.clone(), snapshot.clone())
                        .label(snapshot.label.clone())
                })
                .collect::<Vec<_>>()
        };
        let items: Rc<[CollectionItem<RowSnapshot>]> = items.into();
        let selection = self.selection_binding();
        let selected_label = self
            .selected()
            .as_ref()
            .map_or("None", Identity::as_str)
            .to_owned();
        let count = items.len();
        let reverse_label = if self.reversed() {
            "Restore order"
        } else {
            "Reverse order"
        };
        let remove_disabled = self.selected().is_none();
        let restore_disabled = self.removed().is_none();
        let confirm_count = self.confirm_count();
        let revision = self.diagnostic_refresh();
        let lifecycle = {
            let store = self.rows.borrow();
            format!(
                "Lifecycle snapshot · active {} · mounts {} · releases {} · refresh {}",
                store.lifecycle.active.len(),
                store.lifecycle.mounts,
                store.lifecycle.releases,
                revision,
            )
        };

        OwnerActionReader(move |parent: OwnerAction<CollectionGallery>| {
            let row_store = rows.clone();
            let collection = Collection::new(
                "collection-gallery-rows",
                items.clone(),
                390.0,
                68.0,
                move |_, snapshot: &RowSnapshot| {
                    Component("collection-gallery-row", CollectionRow::default).props(
                        RowProps {
                            snapshot: snapshot.clone(),
                            store: row_store.clone(),
                        },
                        CollectionRow::apply_props,
                    )
                },
            )
            .selection(selection.clone())
            .label("Collection gallery records")
            .on_confirm(parent.action(CollectionGallery::confirm_selected));

            VStack::new()
                .spacing(8.0)
                .child(Text("Collection · virtualized stable-ID rows · 1,000 records"))
                .child(
                    HStack::new()
                        .spacing(6.0)
                        .child(Button(reverse_label, parent.action(CollectionGallery::reverse_order)).id("collection-reverse"))
                        .child(Button("Remove selected", parent.action(CollectionGallery::remove_selected)).id("collection-remove").disabled(remove_disabled))
                        .child(Button("Restore removed", parent.action(CollectionGallery::restore_removed)).id("collection-restore").disabled(restore_disabled))
                        .child(Button("Select last row", parent.action(CollectionGallery::select_last)).id("collection-select-last"))
                        .child(Button("Clear selection", parent.action(CollectionGallery::clear_selection)).id("collection-clear-selection"))
                        .child(Button("Refresh lifecycle", parent.action(CollectionGallery::refresh_diagnostics)).id("collection-refresh-lifecycle")),
                )
                .child(Text(format!(
                    "Rows: {count} · selected ID: {selected_label} · Enter confirmations: {confirm_count}"
                )))
                .child(Text(lifecycle.clone()).font_size(11.0))
                .child(collection)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(values: &[&str]) -> Vec<Identity> {
        values.iter().map(|value| Identity::from(*value)).collect()
    }

    #[test]
    fn restore_uses_the_next_stable_id_anchor_without_reordering() {
        let order = ids(&["a", "c", "d"]);
        let removed = RemovedRow {
            id: Identity::from("b"),
            previous: Some(Identity::from("a")),
            next: Some(Identity::from("c")),
            index: 1,
            reversed: false,
        };

        let index = restored_index(&order, &removed, false);

        assert_eq!(index, 1);
    }

    #[test]
    fn restore_tracks_the_removed_row_when_the_collection_is_reversed() {
        let order = ids(&["d", "c", "a"]);
        let removed = RemovedRow {
            id: Identity::from("b"),
            previous: Some(Identity::from("a")),
            next: Some(Identity::from("c")),
            index: 1,
            reversed: false,
        };

        let index = restored_index(&order, &removed, true);
        let mut restored = order;
        restored.insert(index, removed.id);

        assert_eq!(restored, ids(&["d", "c", "b", "a"]));
    }

    #[test]
    fn restore_uses_the_surviving_anchor_or_clamped_index() {
        let removed = RemovedRow {
            id: Identity::from("b"),
            previous: Some(Identity::from("a")),
            next: Some(Identity::from("c")),
            index: 9,
            reversed: false,
        };

        assert_eq!(restored_index(&ids(&["a", "d"]), &removed, false), 1);
        assert_eq!(restored_index(&ids(&["d"]), &removed, false), 1);
    }
}
