Helps automate mass-disabling of packages in Stackage's build-constraint.yaml.

## Install

* Install rustup from [https://rustup.rs/](https://rustup.rs/)
* Run `cargo install --path .`

## Usage
See [CURATORS.md](https://github.com/commercialhaskell/stackage/blob/master/CURATORS.md).

## Development

* Incremental build: `cargo watch` (and e.g. `cargo watch -x test` to run tests)
* Build: `cargo build`
* Test: `cargo test`
* Linting: `cargo clippy`
* Formatting: `cargo fmt`
