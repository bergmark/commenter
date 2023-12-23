#!/bin/bash

set -euxo pipefail

cargo check
cargo test -q
cargo clippy
cargo machete
cargo fmt -- --check
