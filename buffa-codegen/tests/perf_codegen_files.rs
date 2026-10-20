//! Opt-in end-to-end benchmark for large descriptor-set generation.
//! Run: cargo test -p buffa-codegen --release --test perf_codegen_files -- --ignored --nocapture

use buffa_codegen::generated::descriptor::FileDescriptorProto;
use buffa_codegen::{generate, CodeGenConfig};
use std::hint::black_box;
use std::time::Instant;

fn fixture(n: usize) -> (Vec<FileDescriptorProto>, Vec<String>) {
    let names: Vec<String> = (0..n).map(|i| format!("bench/file_{i:05}.proto")).collect();
    let descriptors = names
        .iter()
        .map(|name| FileDescriptorProto {
            name: Some(name.clone()),
            package: Some("benchmark".to_string()),
            syntax: Some("proto3".to_string()),
            ..Default::default()
        })
        .collect();
    (descriptors, names)
}

#[test]
#[ignore = "opt-in benchmark"]
fn benchmark_large_file_selection() {
    let mut config = CodeGenConfig::default();
    config.generate_views = false;

    for n in [16usize, 128, 512, 2048, 4096] {
        let (files, names) = fixture(n);
        let output = generate(&files, &names, &config).expect("generation");
        assert!(!output.is_empty());

        let iterations = match n {
            16 => 30,
            128 => 10,
            512 => 3,
            2048 => 2,
            _ => 1,
        };
        for _ in 0..2 {
            black_box(generate(black_box(&files), black_box(&names), &config).unwrap());
        }
        let mut samples = Vec::with_capacity(5);
        for _ in 0..5 {
            let start = Instant::now();
            for _ in 0..iterations {
                black_box(generate(black_box(&files), black_box(&names), &config).unwrap());
            }
            samples.push(start.elapsed().as_secs_f64() * 1000.0 / iterations as f64);
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "PERF codegen nfiles={} median_ms_per_generate={:.3} min_ms={:.3} max_ms={:.3} samples={}",
            n,
            samples[2],
            samples[0],
            samples[4],
            samples.len()
        );
    }
}
