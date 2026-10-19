// Standalone textproto escape benchmark. Includes the original character-by-
// character writers as in-process baselines, without modifying the library.
// Run with:
// cargo bench --manifest-path benchmarks/buffa/Cargo.toml --features text_escape_spans --bench text_escape_spans
//
// Not part of the cross-implementation baseline suite.
use buffa::text::{escape_bytes, escape_str};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fmt::Write;

// Reference implementations from before the batched-span change.
fn scalar_escape_bytes<W: Write + ?Sized>(bytes: &[u8], w: &mut W) -> std::fmt::Result {
    w.write_char('"')?;
    for &b in bytes {
        match b {
            b'"' => w.write_str("\\\"")?,
            b'\\' => w.write_str("\\\\")?,
            b'\n' => w.write_str("\\n")?,
            b'\r' => w.write_str("\\r")?,
            b'\t' => w.write_str("\\t")?,
            0x20..=0x7E => w.write_char(b as char)?,
            _ => {
                w.write_char('\\')?;
                w.write_char((b'0' + (b >> 6)) as char)?;
                w.write_char((b'0' + ((b >> 3) & 7)) as char)?;
                w.write_char((b'0' + (b & 7)) as char)?;
            }
        }
    }
    w.write_char('"')
}

fn scalar_escape_str<W: Write + ?Sized>(s: &str, w: &mut W) -> std::fmt::Result {
    w.write_char('"')?;
    for c in s.chars() {
        match c {
            '"' => w.write_str("\\\"")?,
            '\\' => w.write_str("\\\\")?,
            '\n' => w.write_str("\\n")?,
            '\r' => w.write_str("\\r")?,
            '\t' => w.write_str("\\t")?,
            '\x00'..='\x1F' | '\x7F' => {
                let b = c as u8;
                w.write_char('\\')?;
                w.write_char((b'0' + (b >> 6)) as char)?;
                w.write_char((b'0' + ((b >> 3) & 7)) as char)?;
                w.write_char((b'0' + (b & 7)) as char)?;
            }
            _ => w.write_char(c)?,
        }
    }
    w.write_char('"')
}

fn bench_escape_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_escape_bytes");
    let mixed: Vec<u8> = (0..4096)
        .map(|i| match i % 64 {
            0 => b'"',
            1 => b'\\',
            2 => b'\n',
            3 => 0xFF,
            _ => b'a' + (i % 26) as u8,
        })
        .collect();
    let cases = [
        ("ascii_64", b'a'.repeat(64)),
        ("ascii_4096", b'a'.repeat(4096)),
        ("ascii_65536", b'a'.repeat(65536)),
        ("mixed_4096", mixed),
        ("binary_4096", (0u8..=255).cycle().take(4096).collect()),
        ("escaped_4096", b"\"\\\n\t".repeat(1024)),
    ];
    for (name, input) in cases {
        let mut scalar_result = String::new();
        let mut batched_result = String::new();
        scalar_escape_bytes(&input, &mut scalar_result).unwrap();
        escape_bytes(&input, &mut batched_result).unwrap();
        assert_eq!(scalar_result, batched_result, "byte case: {name}");

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("scalar", name), &input, |b, input| {
            let mut out = String::with_capacity(input.len() * 4 + 2);
            b.iter(|| {
                out.clear();
                scalar_escape_bytes(black_box(input.as_slice()), &mut out).unwrap();
                black_box(out.as_str());
            });
        });
        group.bench_with_input(BenchmarkId::new("batched", name), &input, |b, input| {
            let mut out = String::with_capacity(input.len() * 4 + 2);
            b.iter(|| {
                out.clear();
                escape_bytes(black_box(input.as_slice()), &mut out).unwrap();
                black_box(out.as_str());
            });
        });
    }
    group.finish();
}

fn bench_escape_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_escape_str");
    let cases = [
        ("ascii_64", "a".repeat(64)),
        ("ascii_4096", "a".repeat(4096)),
        ("ascii_65536", "a".repeat(65536)),
        ("unicode_4096", "café 🌟".repeat(512)),
        ("mixed_4096", "normal text \" \\\n\t café 🌟".repeat(256)),
        ("escaped_4096", "\"\\\n\t".repeat(1024)),
    ];
    for (name, input) in cases {
        let mut scalar_result = String::new();
        let mut batched_result = String::new();
        scalar_escape_str(&input, &mut scalar_result).unwrap();
        escape_str(&input, &mut batched_result).unwrap();
        assert_eq!(scalar_result, batched_result, "string case: {name}");

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("scalar", name), &input, |b, input| {
            let mut out = String::with_capacity(input.len() * 4 + 2);
            b.iter(|| {
                out.clear();
                scalar_escape_str(black_box(input.as_str()), &mut out).unwrap();
                black_box(out.as_str());
            });
        });
        group.bench_with_input(BenchmarkId::new("batched", name), &input, |b, input| {
            let mut out = String::with_capacity(input.len() * 4 + 2);
            b.iter(|| {
                out.clear();
                escape_str(black_box(input.as_str()), &mut out).unwrap();
                black_box(out.as_str());
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_escape_bytes, bench_escape_str);
criterion_main!(benches);
