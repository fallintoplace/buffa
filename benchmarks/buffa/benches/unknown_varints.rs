// Benchmark synthetic wire records generated for unrecognized closed-enum
// values, including packed fields that preserve many values per message.
//
// Run from benchmarks/buffa:
//   cargo bench --features unknown_varints --bench unknown_varints
// Compare against main with the same toolchain and release profile.
use std::cell::Cell;

use buffa::{DecodeContext, UnknownFieldsView, RECURSION_LIMIT};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn bench_unknown_varints(c: &mut Criterion) {
    let mut group = c.benchmark_group("view/unknown_varints");

    for &n in &[32usize, 512, 4096] {
        for &(label, field_number, value) in &[
            ("short", 3u32, 127u64),
            ("max_length", (1u32 << 29) - 1, u64::MAX),
        ] {
            group.throughput(Throughput::Elements(n as u64));
            group.bench_with_input(BenchmarkId::new(label, n), &n, |b, &count| {
                b.iter(|| {
                    let remaining = Cell::new(count);
                    let ctx = DecodeContext::new(RECURSION_LIMIT, &remaining);
                    let mut view = UnknownFieldsView::new();
                    for _ in 0..count {
                        view.push_varint(black_box(field_number), black_box(value), ctx)
                            .unwrap();
                    }
                    black_box(view);
                })
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_unknown_varints);
criterion_main!(benches);
