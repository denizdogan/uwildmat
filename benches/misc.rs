use criterion::{
  black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
  Throughput,
};
use uwildmat::regular;

fn misc_regular(c: &mut Criterion) {
  let mut group = c.benchmark_group("regular");
  const LOREM: &str = "lorem ipsum";
  let cases = [
    (LOREM, LOREM, "exact"),
    // *
    (LOREM, "*", "star-only"),
    (LOREM, "lorem*ipsum", "star-one"),
    ("***********", "***********", "star-all"),
    // ?
    ("a", "?", "q-only"),
    (LOREM, "lorem?ipsum", "q-one"),
    (LOREM, "???????????", "q-all"),
    // wide set ranges
    (LOREM, &"[\u{0001}-\u{ffff}]".repeat(11), "set-wide-all"),
    (LOREM, &"[^\u{1111}-\u{ffff}]".repeat(11), "set-wide-all-n"),
    ("\u{7777}", "[\u{1111}-\u{ffff}]", "set-wide-one"),
    ("\u{7777}", "[^\u{1111}-\u{ffff}]", "set-wide-one-n"),
    // slim set ranges
    (LOREM, &"[a-z]".repeat(11), "set-slim-all"),
    (LOREM, &"[^x-z]".repeat(11), "set-slim-all-n"),
    ("m", "[^k-n]", "set-slim-one"),
    ("m", "[^x-z]", "set-slim-one-n"),
  ];
  for (text, pattern, id) in cases.iter() {
    group.throughput(Throughput::Bytes(pattern.len().try_into().unwrap()));
    group.bench_with_input(
      BenchmarkId::from_parameter(id),
      &(text, pattern),
      |bencher, (text, pattern)| {
        bencher.iter(|| regular(black_box(&text), black_box(&pattern)));
      },
    );
  }
  group.bench_function("compare-chars-iter", |b| {
    b.iter(|| {
      black_box("hello world")
        .chars()
        .eq(black_box("hello_world").chars())
    });
  });
  group.finish();
}

criterion_group! {
  name = benches;
  config = Criterion::default().significance_level(0.02).sample_size(1000);
  targets = misc_regular
}

criterion_main!(benches);
