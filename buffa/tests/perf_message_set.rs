//! Opt-in end-to-end benchmark for MessageSet Item payload merging.
//! Run with: cargo test -p buffa --release --test perf_message_set -- --ignored --nocapture

use buffa::encoding::encode_varint;
use buffa::message_set::{merge_item, ITEM_END_TAG, MESSAGE_TAG, TYPE_ID_TAG};
use buffa::{DecodeContext, RECURSION_LIMIT};
use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

fn wire_item(payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(payload.len() + 16);
    encode_varint(TYPE_ID_TAG, &mut buf);
    encode_varint(42, &mut buf);
    encode_varint(MESSAGE_TAG, &mut buf);
    encode_varint(payload.len() as u64, &mut buf);
    buf.extend_from_slice(payload);
    encode_varint(ITEM_END_TAG, &mut buf);
    buf
}

#[test]
#[ignore = "run manually for performance measurements"]
fn benchmark_message_set_merge_item() {
    for len in [64usize, 4096, 65536] {
        let payload = vec![0xa5; len];
        let encoded = wire_item(&payload);
        let remaining = Cell::new(usize::MAX);
        let ctx = DecodeContext::new(RECURSION_LIMIT, &remaining);
        let iterations = match len {
            64 => 100_000,
            4096 => 15_000,
            _ => 2_000,
        };

        // Check that both revisions decode the expected complete payload.
        let mut probe = encoded.as_slice();
        let (id, decoded) = merge_item(&mut probe, ctx).unwrap();
        assert_eq!(id, 42);
        assert_eq!(decoded, payload);
        assert!(probe.is_empty());

        for _ in 0..100 {
            let mut src = black_box(encoded.as_slice());
            black_box(merge_item(&mut src, ctx).unwrap());
        }

        let mut samples = Vec::with_capacity(9);
        for _ in 0..9 {
            let start = Instant::now();
            for _ in 0..iterations {
                let mut src = black_box(encoded.as_slice());
                let (type_id, data) = merge_item(&mut src, ctx).unwrap();
                black_box((type_id, data));
            }
            samples.push(start.elapsed().as_nanos() as f64 / f64::from(iterations));
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "PERF message_set bytes={} median_ns_per_op={:.2} min_ns_per_op={:.2} max_ns_per_op={:.2} iterations={} samples={}",
            len,
            samples[4],
            samples[0],
            samples[8],
            iterations,
            samples.len()
        );
    }
}
