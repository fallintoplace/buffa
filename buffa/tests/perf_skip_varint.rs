//! Opt-in benchmark for skipping unknown protobuf varints.
//! Run: cargo test -p buffa --release --test perf_skip_varint -- --ignored --nocapture

use buffa::encoding::{encode_varint, skip_field_depth, Tag, WireType};
use buffa::RECURSION_LIMIT;
use std::hint::black_box;
use std::time::Instant;

fn encoded_fields(value: u64) -> Vec<u8> {
    let mut data = Vec::new();
    for _ in 0..4096 {
        Tag::new(1, WireType::Varint).encode(&mut data);
        encode_varint(value, &mut data);
    }
    data
}

#[test]
#[ignore = "opt-in benchmark"]
fn benchmark_skip_unknown_varints() {
    for (label, value) in [
        ("one_byte", 1u64),
        ("two_bytes", 300),
        ("five_bytes", u32::MAX as u64),
        ("ten_bytes", u64::MAX),
    ] {
        let wire = encoded_fields(value);
        let repetitions = 1000u32;
        // Verify that all 4096 fields are consumed.
        let mut probe = wire.as_slice();
        for _ in 0..4096 {
            let tag = Tag::decode(&mut probe).unwrap();
            skip_field_depth(tag, &mut probe, RECURSION_LIMIT).unwrap();
        }
        assert!(probe.is_empty());

        for _ in 0..20 {
            let mut src = black_box(wire.as_slice());
            while !src.is_empty() {
                let tag = Tag::decode(&mut src).unwrap();
                skip_field_depth(tag, &mut src, RECURSION_LIMIT).unwrap();
            }
            black_box(src);
        }

        let mut samples = Vec::with_capacity(7);
        for _ in 0..7 {
            let started = Instant::now();
            for _ in 0..repetitions {
                let mut src = black_box(wire.as_slice());
                while !src.is_empty() {
                    let tag = Tag::decode(&mut src).unwrap();
                    skip_field_depth(tag, &mut src, RECURSION_LIMIT).unwrap();
                }
                black_box(src);
            }
            let ns_per_field =
                started.elapsed().as_nanos() as f64 / (f64::from(repetitions) * 4096.0);
            samples.push(ns_per_field);
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "PERF skip_varint case={} median_ns_per_field={:.3} min_ns_per_field={:.3} max_ns_per_field={:.3} samples={}",
            label,
            samples[3],
            samples[0],
            samples[6],
            samples.len()
        );
    }
}
