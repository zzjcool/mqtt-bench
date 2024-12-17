use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn criterion_benchmark(c: &mut Criterion) {
    let mut ca_key_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ca_key_path_buf.push("assets");
    ca_key_path_buf.push("CA.key");
    let ca_key = mqtt_bench::cert::load_ca_pkey(&ca_key_path_buf).unwrap();

    let mut ca_cert_path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ca_cert_path_buf.push("assets");
    ca_cert_path_buf.push("CA.crt");
    let ca_cert = mqtt_bench::cert::load_ca_cert(&ca_cert_path_buf).unwrap();

    let mut seq = 0;

    c.bench_function("SignX509Certs", |b| {
        b.iter(|| {
            let common_name = format!("common-name-{}", seq);
            seq += 1;
            mqtt_bench::cert::mk_ca_signed_cert(&ca_cert, &ca_key, &common_name)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
