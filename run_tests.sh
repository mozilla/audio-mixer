# Bail out once getting an error.
set -e

# Display verbose backtrace for debugging if backtrace is unset
if [ -z "${RUST_BACKTRACE}" ]; then
  export RUST_BACKTRACE=1
fi
echo "RUST_BACKTRACE is set to ${RUST_BACKTRACE}\n"

# Format check
cargo fmt -- --check

# Lints check
cargo clippy --all-targets --all-features -- -D warnings

# Regular Tests
# We need to run `cargo clean` before the `cargo test`. Otherwise, we will get a
# "multiple matching crates for `bitflags`" error when running `test_build_ffi`
# since `cargo clippy` already build a `bitflags` crate and `cargo test` will
# build another one again.
cargo clean
cargo test --verbose --all-features
