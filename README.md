liboffsetup
===========
[![Build Status](https://travis-ci.org/offscale/liboffsetup.svg?branch=master)](https://travis-ci.org/offscale/liboffsetup)

Offsetup bootstraps nodes. Unwraps Docker.
Cross-platform focus: Windows, Linux and macOS.

## RFCs

Of interest is its RFC: https://offsetup.offscale.io

## Developer guide

Install the latest version of [Rust](https://www.rust-lang.org). We tend to use nightly versions. [CLI tool for installing Rust](https://rustup.rs).

We use [rust-clippy](https://github.com/rust-lang-nursery/rust-clippy) linters to improve code quality.

There are plenty of [IDEs](https://areweideyet.com) and other [Rust development tools to consider](https://github.com/rust-unofficial/awesome-rust#development-tools).

### Step-by-step guide

```bash
# Install Rust (nightly)
$ curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
# Install cargo-make (cross-platform feature-rich reimplementation of Make)
$ cargo install --force cargo-make
# Install rustfmt (Rust formatter)
$ rustup component add rustfmt
# Clone this repo
$ git clone https://github.com/offscale/liboffsetup && cd liboffsetup
# Run tests
$ cargo test
# Format, build and test
$ cargo make
```

### Notes for Windows
Tested/compiled with nightly-x86_64-pc-windows-msvc and nightly-x86_64-pc-windows-gnu. 
Under msys2, make sure that the following is in the **.cargo/config** file (update the paths if necessary):
```
[target.x86_64-pc-windows-gnu]
linker = "C://msys64//mingw64//bin/gcc.exe"
ar = "C://msys64//mingw64//bin//ar.exe"
``` 

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
