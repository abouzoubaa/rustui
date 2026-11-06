use rustui::component;

#[component]
struct Counter {
    #[state(0)]
    value: u32,
}

#[test]
fn generated_state_setter_reports_only_changes() {
    let mut counter = Counter::default();
    assert_eq!(counter.value(), 0);
    assert!(!counter.set_value(0));
    assert_eq!(counter.value(), 0);
    assert!(counter.set_value(1));
    assert_eq!(counter.value(), 1);

    let state = counter.value_state();
    assert_eq!(state.get(), 1);
    let binding = state.binding();
    drop(state);

    assert_eq!(binding.read_from(&counter), Some(1));
    assert!(binding.write_to(&mut counter, 2));
    assert_eq!(counter.value(), 2);
}

#[test]
fn generated_state_updater_returns_closure_output_and_updates_field() {
    let mut counter = Counter::default();

    let output = counter.update_value(|value| {
        *value += 2;
        *value
    });

    assert_eq!(output, 2);
    assert_eq!(counter.value(), 2);
}
