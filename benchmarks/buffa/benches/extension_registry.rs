// Opt-in benchmark for ProtoJSON extension serialization with sparse
// extension registrations. Run:
// cargo bench --manifest-path benchmarks/buffa/Cargo.toml \
//   --bench extension_registry --features extension_registry
//
// Benchmarks use a non-empty global registry throughout. The message varies
// by whether its extendee has a registry bucket and whether unknown fields
// match a registered number. Setup is outside timed iterations.

use buffa::extension_registry::{serialize_extensions, ExtensionRegistry, JsonExtEntry};
use buffa::{UnknownField, UnknownFieldData, UnknownFields};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::Serialize;

struct ExtensionJson<'a> {
    extendee: &'static str,
    fields: &'a UnknownFields,
}

impl Serialize for ExtensionJson<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_extensions(self.extendee, self.fields, serializer)
    }
}

#[allow(deprecated)]
fn register_extensions() {
    let mut registry = ExtensionRegistry::new();
    registry.register(JsonExtEntry {
        number: 50,
        full_name: "pkg.benchmark",
        extendee: "pkg.Benchmark",
        to_json: buffa::extension_registry::helpers::int32_to_json,
        from_json: buffa::extension_registry::helpers::int32_from_json,
    });
    buffa::extension_registry::set_extension_registry(Box::new(registry));
}

fn fields(count: u32, registered: bool) -> UnknownFields {
    let mut out = UnknownFields::new();
    for number in 1000..1000 + count {
        out.push(UnknownField {
            number,
            data: UnknownFieldData::Varint(u64::from(number)),
        });
    }
    if registered {
        out.push(UnknownField {
            number: 50,
            data: UnknownFieldData::Varint(7),
        });
        out.push(UnknownField {
            number: 50,
            data: UnknownFieldData::Varint(9),
        });
    }
    out
}

fn bench_extension_registry(c: &mut Criterion) {
    register_extensions();
    let many_unregistered = fields(4096, false);
    let sparse = fields(4096, true);
    let tiny = fields(0, true);

    let mut group = c.benchmark_group("extension_json/serialize");
    for (name, extendee, data) in [
        ("no_extendee_bucket_4096", "pkg.NoExtensions", &many_unregistered),
        ("no_registered_number_4096", "pkg.Benchmark", &many_unregistered),
        ("one_registered_number_4096", "pkg.Benchmark", &sparse),
        ("tiny_two_records", "pkg.Benchmark", &tiny),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(
                    serde_json::to_string(&ExtensionJson {
                        extendee,
                        fields: black_box(data),
                    })
                    .unwrap(),
                )
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_extension_registry);
criterion_main!(benches);
