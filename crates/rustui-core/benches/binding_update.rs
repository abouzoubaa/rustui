//! Owner-bound update cost only; this does not measure GPUI notifications or rendering.
use rustui_core::Binding;
use std::{hint::black_box, time::Instant};

#[derive(Default)]
struct Owner {
    value: u64,
    writes: u64,
}

fn run(iterations: u64, changed: bool) -> u128 {
    let mut owner = Owner::default();
    let binding = Binding::new(
        0,
        |owner: &Owner| owner.value,
        |owner: &mut Owner, value| {
            owner.value = value;
            owner.writes += 1;
        },
    );
    let start = Instant::now();
    for _ in 0..iterations {
        let result = binding.update_on(black_box(&mut owner), |value| {
            if black_box(changed) {
                *value += 1;
                true
            } else {
                false
            }
        });
        black_box(result);
    }
    let elapsed = start.elapsed().as_nanos();
    let expected = if changed { iterations } else { 0 };
    assert_eq!((owner.value, owner.writes), (expected, expected));
    elapsed
}

fn main() {
    let iterations = std::env::var("RUSTUI_BENCH_ITERATIONS")
        .ok()
        .map(|value| value.parse::<u64>().expect("iterations must be an integer"))
        .unwrap_or(1_000_000);
    assert!((1..=10_000_000).contains(&iterations));
    let warmup = (iterations / 10).max(1);
    for changed in [false, true] {
        black_box(run(warmup, changed));
    }
    println!("workload,sample,iterations,elapsed_ns,ns_per_update");
    for sample in 1..=10 {
        // Alternate order so a systematic warm-up or thermal trend favors neither case.
        for changed in if sample % 2 == 0 {
            [true, false]
        } else {
            [false, true]
        } {
            let elapsed = run(iterations, changed);
            let name = if changed { "changed" } else { "unchanged" };
            println!(
                "{name},{sample},{iterations},{elapsed},{:.3}",
                elapsed as f64 / iterations as f64
            );
        }
    }
}
