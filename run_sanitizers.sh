# The option `Z` is only accepted on the nightly compiler
# so changing to nightly toolchain by `rustup default nightly` is required.

# See: https://github.com/rust-lang/rust/issues/39699 for more sanitizer support.

toolchain="$(rustup default)"
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
mv Cargo.toml $ORIGINAL_MANIFEST
# Delete lines that contain a `criterion` from Cargo.toml.
sed '/criterion/d' $ORIGINAL_MANIFEST > Cargo.toml

sanitizers=("address" "leak" "memory" "thread")
for san in "${sanitizers[@]}"
do
  San="$(tr '[:lower:]' '[:upper:]' <<< ${san:0:1})${san:1}"
  echo "\n\nRun ${San}Sanitizer\n------------------------------"
  if [ $san = "leak" ]; then
    echo "Skip the test that panics. Leaking memory when the program drops out abnormally is ok."
    options="-- --Z unstable-options --exclude-should-panic"
  fi
  # We need to run `cargo clean` before the `cargo test`. Otherwise, we will get a
  # "multiple matching crates for `audio_mixer`" error when running `test_build_ffi` since
  # `audio_mixer` crate is built before (e.g., previous test round) and `cargo test` will
  # build another one again.
  cargo clean
  RUSTFLAGS="-Z sanitizer=${san}" cargo test $options
done

mv $ORIGINAL_MANIFEST Cargo.toml