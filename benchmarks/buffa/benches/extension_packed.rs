//! Opt-in packed-extension allocation benchmark.
//!
//! Run:
//! cargo bench --manifest-path benchmarks/buffa/Cargo.toml --no-default-features --features packed_extension --bench extension_packed
//!
//! Compare the same benchmark IDs on upstream main and this branch.
use buffa::extension::codecs::{Fixed32, Fixed64, Int32, PackedRepeated};
use buffa::extension::ExtensionCodec;
use buffa::unknown_fields::{UnknownField, UnknownFieldData, UnknownFields};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};

fn packed_field(data: Vec<u8>) -> UnknownFields {
    let mut fields = UnknownFields::new();
    fields.push(UnknownField {
        number: 1,
        data: UnknownFieldData::LengthDelimited(data),
    });
    fields
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("extension_packed_decode");
    for &n in &[8usize, 128, 4_096] {
        let fields32 = packed_field(
            (0..n as u32)
                .flat_map(|v| v.to_le_bytes())
                .collect(),
        );
        let fields64 = packed_field(
            (0..n as u64)
                .flat_map(|v| v.to_le_bytes())
                .collect(),
        );
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("fixed32", n), &fields32, |b, fields| {
            b.iter(|| black_box(PackedRepeated::<Fixed32>::decode(1, black_box(fields))));
        });
        group.bench_with_input(BenchmarkId::new("fixed64", n), &fields64, |b, fields| {
            b.iter(|| black_box(PackedRepeated::<Fixed64>::decode(1, black_box(fields))));
        });
    }
    group.finish();
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("extension_packed_encode");
    for &n in &[8usize, 128, 4_096] {
        let fixed32: Vec<u32> = (0..n as u32).collect();
        let fixed64: Vec<u64> = (0..n as u64).collect();
        let varint: Vec<i32> = (0..n as i32).collect();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("fixed32", n), &fixed32, |b, values| {
            b.iter_batched(
                || (*values).clone(),
                |input| {
                    let mut fields = UnknownFields::new();
                    PackedRepeated::<Fixed32>::try_encode(1, black_box(input), &mut fields).unwrap();
                    black_box(fields);
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("fixed64", n), &fixed64, |b, values| {
            b.iter_batched(
                || (*values).clone(),
                |input| {
                    let mut fields = UnknownFields::new();
                    PackedRepeated::<Fixed64>::try_encode(1, black_box(input), &mut fields).unwrap();
                    black_box(fields);
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("varint", n), &varint, |b, values| {
            b.iter_batched(
                || (*values).clone(),
                |input| {
                    let mut fields = UnknownFields::new();
                    PackedRepeated::<Int32>::try_encode(1, black_box(input), &mut fields).unwrap();
                    black_box(fields);
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_decode, bench_encode);
criterion_main!(benches);
