# Bail out once getting an error.
set -e

# Display verbose backtrace for debugging if backtrace is unset
if [ -z "${RUST_BACKTRACE}" ]; then
  export RUST_BACKTRACE=1
fi
echo "RUST_BACKTRACE is set to ${RUST_BACKTRACE}\n"

# Regular Tests
echo "\ncargo test\n----------"
cargo test --workspace --verbose

# Format check
echo "cargo fmt\n----------"
cargo fmt --all -- --check

# Lints check
echo "\ncargo clippy\n----------"
cargo clippy --workspace -- -D warnings
