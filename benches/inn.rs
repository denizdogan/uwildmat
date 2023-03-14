use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uwildmat::{poison, regular, simple, Uwildmat};

include!(concat!(env!("OUT_DIR"), "/gen_test_suite.rs"));

// test_r in INN
#[inline(always)]
fn bench_regular(_n: usize, value: &str, pattern: &str, _expected: bool) {
  regular(black_box(&value), black_box(&pattern));
}

// test_p in INN
#[inline(always)]
fn bench_poison(_n: usize, value: &str, pattern: &str, _expected: Uwildmat) {
  poison(black_box(&value), black_box(&pattern));
}

// test_s in INN
#[inline(always)]
fn bench_simple(_n: usize, value: &str, pattern: &str, _expected: bool) {
  simple(black_box(&value), black_box(&pattern));
}

pub fn inn(c: &mut Criterion) {
  c.bench_function("inn test suite", |b| {
    b.iter(|| {
      run_inn_test_suite(bench_regular, bench_poison, bench_simple);
    })
  });
}

criterion_group! {
  name = benches;
  config = Criterion::default().significance_level(0.02).sample_size(1000);
  targets = inn
}

criterion_main!(benches);
