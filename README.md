# uwildmat

uwildmat implemented in Rust.

No dependencies.

Heavily based on the [original C version][uwildmat] of uwildmat, which is part
of [InterNetNews][inn] (INN).

## Usage

```toml
# Cargo.toml
[dependencies]
uwildmat = "0.2"
```

```rust
// your_code.rs
use uwildmat::simple as uwildmat;

let text = "foo";
let pattern = "*foo*";
asserteq!(uwildmat(text, pattern), true);

// or:

use uwildmat::regular as uwildmat;

let text = "foo";
let pattern = "*foo*";
asserteq!(uwildmat(text, pattern), true);

// or:

use uwildmat::poison as uwildmat;

let text = "foo";
let pattern = "*foo*";
asserteq!(uwildmat(text, pattern), true);
```

## Differences from INN's uwildmat

This module will _not_ handle invalid UTF-8. INN's uwildmat will (technically)
allow any byte sequences as input, even if it is invalid UTF-8.

## Development

Use the nightly toolchain:

```console
$ rustup override set nightly
```

Then the usual:

```console
$ cargo clean
$ cargo build
$ cargo test
$ cargo bench
```

### Benchmarks

When making changes, always check how much it affects the performance first.

Before making changes, get a baseline of the "misc" benchmark group:

```console
$ cargo bench --bench misc -- --save-baseline before
```

After making changes, compare the performance against the baseline:

```console
$ cargo bench --bench misc -- --baseline before
```

### Build script

The build script [`src/build.rs`](src/build.rs) downloads the INN uwildmat test
suite and processes it, generating an equivalent Rust test suite.

[uwildmat]: https://github.com/InterNetNews/inn/blob/main/lib/uwildmat.c
[inn]: https://github.com/InterNetNews/inn/tree/main
