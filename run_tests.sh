# Bail out once getting an error.
set -e

# Display verbose backtrace for debugging if backtrace is unset
if [ -z "${RUST_BACKTRACE}" ]; then
  export RUST_BACKTRACE=1
fi
echo "RUST_BACKTRACE is set to ${RUST_BACKTRACE}\n"

# Regular Tests
cargo test --verbose --all-features

# Format check
cargo fmt -- --check

# Lints check
cargo clippy --all-targets --all-features -- -D warnings
