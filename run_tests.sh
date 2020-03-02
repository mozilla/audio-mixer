# Bail out once getting an error.
set -e

# Display verbose backtrace for debugging if backtrace is unset
if [ -z "${RUST_BACKTRACE}" ]; then
  export RUST_BACKTRACE=1
fi
echo "RUST_BACKTRACE is set to ${RUST_BACKTRACE}\n"

# We need to run `cargo clean` before the `cargo test`. Otherwise, we may get a
# "multiple matching crates for `audio_mixer`" error when running `test_build_ffi` since
# `audio_mixer` crate may be built before (e.g., previous test round) and `cargo test` will
# build another one again.
cargo clean

# Regular Tests
echo "\ncargo test\n----------"
cargo test --workspace --verbose

# Format check
echo "cargo fmt\n----------"
cargo fmt --all -- --check

# Lints check
echo "\ncargo clippy\n----------"
cargo clippy --workspace -- -D warnings
