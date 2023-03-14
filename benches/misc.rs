use criterion::{
  black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use uwildmat::simple;

const LOREM: &str = &"lorem ipsum dolor sit amet";

struct Input {
  value: String,
  pattern: String,
  name: String,
}

impl std::fmt::Display for Input {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)
  }
}

impl Input {
  fn new(value: &str, pattern: &str, name: &str) -> Self {
    Self {
      value: value.to_string(),
      pattern: pattern.to_string(),
      name: name.to_string(),
    }
  }
}

fn misc(c: &mut Criterion) {
  let mut group = c.benchmark_group("simple");
  let cases = [
    Input::new(LOREM, LOREM, "exact match"),
    Input::new(LOREM, "*", "single *"),
    Input::new(LOREM, "??????????????????????????", "all ?"),
    Input::new(LOREM, "l?r?m?i?s?m?d?l?r?s?t?a?e?", "every other ?"),
    Input::new(LOREM, "?????????????olor sit amet", "first half ?"),
    Input::new(LOREM, "lorem ipsum d?????????????", "second half ?"),
  ];
  for case in cases.iter() {
    // simple_group.throughput(Throughput::Bytes(*size as u64));
    group.bench_with_input(BenchmarkId::from_parameter(case), case, |b, c| {
      b.iter(|| simple(black_box(&c.value), black_box(&c.pattern)));
    });
  }
  group.finish();
}

criterion_group!(benches, misc);
criterion_main!(benches);
