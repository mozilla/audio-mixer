# The option `Z` is only accepted on the nightly compiler
# so changing to nightly toolchain by `rustup default nightly` is required.

# See: https://github.com/rust-lang/rust/issues/39699 for more sanitizer support.

toolchain=$(rustup default)
echo "\nUse Rust toolchain: $toolchain"

NIGHTLY_PREFIX="nightly"
if [ ! -z "${toolchain##$NIGHTLY_PREFIX*}" ]; then
  echo "The sanitizer is only available on Rust Nightly only. Skip."
  exit
fi

# Display verbose backtrace for debugging if backtrace is unset
if [ -z "${RUST_BACKTRACE}" ]; then
  export RUST_BACKTRACE=1
fi
echo "RUST_BACKTRACE is set to ${RUST_BACKTRACE}\n"

# {Address, Thread}Sanitizer cannot build with `criterion` crate, so `criterion` will be removed
# from the `Cargo.toml` temporarily during the sanitizer tests.
ORIGINAL_MANIFEST="Cargo_ori.toml"
TEST_MANIFEST="Cargo.toml"
mv $TEST_MANIFEST $ORIGINAL_MANIFEST
# Delete lines that contain a `criterion` from Cargo.toml.
sed '/criterion/d' $ORIGINAL_MANIFEST > $TEST_MANIFEST

sanitizers=("address" "leak" "memory" "thread")
for san in "${sanitizers[@]}"
do
  San="$(tr '[:lower:]' '[:upper:]' <<< ${san:0:1})${san:1}"
  echo "\n\nRun ${San}Sanitizer\n------------------------------"
  if [ $san = "leak" ]; then
    echo "Skip the test that panics. Leaking memory when the program drops out abnormally is ok."
    options="-- --Z unstable-options --exclude-should-panic"
  fi
  RUSTFLAGS="-Z sanitizer=${san}" cargo test $options
done

mv $ORIGINAL_MANIFEST $TEST_MANIFEST
