use criterion::{black_box, criterion_group, criterion_main, Criterion};

use uwildmat::{poison, regular, simple, Uwildmat};

include!(concat!(env!("OUT_DIR"), "/gen_test_suite.rs"));

// test_r in INN
fn bench_regular(_n: usize, value: &str, pattern: &str, _expected: bool) {
  regular(&value, &pattern);
}

// test_p in INN
fn bench_poison(_n: usize, value: &str, pattern: &str, _expected: Uwildmat) {
  poison(&value, &pattern);
}

// test_s in INN
fn bench_simple(_n: usize, value: &str, pattern: &str, _expected: bool) {
  simple(&value, &pattern);
}

fn bench_validate(_n: usize, _value: &str, _expected: bool) {}

pub fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("inn test suite", |b| {
    b.iter(|| {
      black_box(run_inn_test_suite(
        bench_regular,
        bench_poison,
        bench_simple,
        bench_validate,
      ));
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
