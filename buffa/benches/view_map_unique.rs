// Direct A/B microbenchmark for MapView::iter_unique.
// "before" copies the original last_occurrence_indices implementation.
// Run: cargo bench -p buffa --bench view_map_unique
use buffa::MapView;
use std::hint::black_box;
use std::time::Instant;

const SAMPLES: usize = 9;

fn before(entries: &[(u32, u32)]) -> usize {
    let n = entries.len();
    let order: Vec<usize> = if n <= 1 {
        (0..n).collect()
    } else {
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_unstable_by(|&a, &b| entries[a].0.cmp(&entries[b].0).then_with(|| a.cmp(&b)));
        let mut kept = 0usize;
        for r in 0..n {
            let last_of_run =
                r + 1 == n || entries[order[r + 1]].0.cmp(&entries[order[r]].0).is_ne();
            if last_of_run {
                order[kept] = order[r];
                kept += 1;
            }
        }
        order.truncate(kept);
        order.sort_unstable();
        order
    };
    order.into_iter().fold(0usize, |acc, i| {
        acc.wrapping_add((entries[i].0 as usize).wrapping_mul(31) ^ entries[i].1 as usize)
    })
}

fn after(map: &MapView<'_, u32, u32>) -> usize {
    map.iter_unique().fold(0usize, |acc, &(key, value)| {
        acc.wrapping_add((key as usize).wrapping_mul(31) ^ value as usize)
    })
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn measure(rounds: usize, mut run: impl FnMut() -> usize) -> f64 {
    let mut checksum = 0usize;
    let start = Instant::now();
    for _ in 0..rounds {
        checksum = checksum.wrapping_add(black_box(run()));
    }
    black_box(checksum);
    start.elapsed().as_secs_f64() * 1e9 / rounds as f64
}

fn bench(name: &str, entries: Vec<(u32, u32)>) {
    let map = MapView::new(entries.clone());
    assert_eq!(before(&entries), after(&map), "{name}: results differ");
    let n = entries.len();
    let rounds = if n <= 8 {
        100_000
    } else if n <= 64 {
        10_000
    } else {
        500
    };

    let mut old = Vec::new();
    let mut new = Vec::new();
    for sample in 0..SAMPLES {
        // Interleave revisions to reduce scheduler / thermal order bias.
        if sample % 2 == 0 {
            old.push(measure(rounds, || before(black_box(&entries))));
            new.push(measure(rounds, || after(black_box(&map))));
        } else {
            new.push(measure(rounds, || after(black_box(&map))));
            old.push(measure(rounds, || before(black_box(&entries))));
        }
    }

    let base = median(&mut old);
    let head = median(&mut new);
    println!(
        "{name:<20} n={n:<4} before={base:>9.2} ns/op  after={head:>9.2} ns/op  speedup={:.2}x",
        base / head
    );
}

fn main() {
    println!("MapView::iter_unique, release, {} interleaved samples; includes index allocation and traversal", SAMPLES);
    let mk = |len: u32, modulo: u32| -> Vec<(u32, u32)> {
        (0..len)
            .map(|i| ((i.wrapping_mul(7) + 3) % modulo, i))
            .collect()
    };
    bench("one", mk(1, 1));
    bench("two-unique", mk(2, 2));
    bench("two-duplicate", mk(2, 1));
    bench("three-unique", mk(3, 3));
    bench("three-duplicate", mk(3, 1));
    bench("four-unique", mk(4, 4));
    bench("four-two-keys", mk(4, 2));
    bench("four-duplicate", mk(4, 1));
    bench("five-unique-control", mk(5, 5));
    bench("eight-unique-control", mk(8, 8));
    bench("eight-repeat-control", mk(8, 2));
    bench("64-unique-control", mk(64, 64));
    bench("64-repeat-control", mk(64, 4));
    bench("512-unique-control", mk(512, 512));
}
