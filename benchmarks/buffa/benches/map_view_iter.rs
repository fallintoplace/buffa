//! Opt-in benchmark for map-view last-write-wins iteration.
//!
//! cargo bench --manifest-path benchmarks/buffa/Cargo.toml --bench map_view_iter --features map_view_iter

use buffa::MapView;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn map_view_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_view_iter_unique");
    for n in [64usize, 512, 4096, 16384] {
        let ordered = MapView::new((0..n).map(|i| (i as u32, i as u32)).collect());
        let runs = MapView::new((0..n).map(|i| ((i / 4) as u32, i as u32)).collect());
        // The multiplier is odd, so it permutes keys for these power-of-two sizes.
        let shuffled = MapView::new(
            (0..n)
                .map(|i| (((i * 8959) % n) as u32, i as u32))
                .collect(),
        );

        for (name, map) in [
            ("ordered_distinct", &ordered),
            ("ordered_runs", &runs),
            ("shuffled_distinct", &shuffled),
        ] {
            group.bench_with_input(BenchmarkId::new(name, n), map, |b, view| {
                // iter_unique computes survivor indexes eagerly.
                b.iter(|| black_box(black_box(view).iter_unique().len()));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, map_view_iter);
criterion_main!(benches);
