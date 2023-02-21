# uwildmat

uwildmat implemented in Rust.

No dependencies.

Heavily based on the [original C version][uwildmat] of uwildmat, which is part
of [InterNetNews][inn] (INN).

## Usage

```toml
[dependencies]
uwildmat = "0.1"
```

```rust
use uwildmat::simple::uwildmat as uwildmat;

let text = "foo";
let pattern = "*foo*";
asserteq!(uwildmat(text, pattern), true);

// or:

use uwildmat::regular::uwildmat as uwildmat;

let text = "foo";
let pattern = "*foo*";
asserteq!(uwildmat(text, pattern), true);

// or:

use uwildmat::poison::uwildmat as uwildmat;

let text = "foo";
let pattern = "*foo*";
asserteq!(uwildmat(text, pattern), true);
```

## Differences from INN's uwildmat

This module will _not_ handle invalid UTF-8. INN's uwildmat will (technically)
allow any byte sequences as input, even if it is invalid UTF-8.

## Development

Remember to use the nightly toolchain for this project:

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

The build script [`src/build.rs`](src/build.rs) downloads the INN uwildmat test
suite and processes it, generating an equivalent Rust test suite.

[uwildmat]: https://github.com/InterNetNews/inn/blob/main/lib/uwildmat.c
[inn]: https://github.com/InterNetNews/inn/tree/main
