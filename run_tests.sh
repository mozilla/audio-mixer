# Bail out once getting an error.
set -e

# Display verbose backtrace for debugging if backtrace is unset
if [ -z "${RUST_BACKTRACE}" ]; then
  export RUST_BACKTRACE=1
fi
echo "RUST_BACKTRACE is set to ${RUST_BACKTRACE}\n"

# Format check
echo "cargo fmt\n----------"
cargo fmt --all -- --check

# Lints check
echo "\ncargo clippy\n----------"
cargo clippy --workspace -- -D warnings

# Regular Tests
echo "\ncargo test\n----------"
# We need to run `cargo clean` before the `cargo test`. Otherwise, we will get a
# "multiple matching crates for `audio_mixer`" error when running `test_build_ffi`
# since `cargo clippy` already build a `audio_mixer` crate and `cargo test` will
# build another one again.
cargo clean
cargo test --workspace --verbose
