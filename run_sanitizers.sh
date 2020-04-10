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
TEST_MANIFEST="Cargo.toml"
mv $TEST_MANIFEST $ORIGINAL_MANIFEST
# Delete lines that contain a `criterion` from Cargo.toml.
sed '/criterion/d' $ORIGINAL_MANIFEST > $TEST_MANIFEST

# Rust API
# -----------------------------------------------------------------------------
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

# C API
# -----------------------------------------------------------------------------
# `cbindgen` cannot work with sanitizers, so we need to avoid building crate
# with `cbindgen` when enabling sanitizers. We can generate the library and
# the header that will be used during the test in advance, and then disable
# the build script, which uses `cbindgen`, to run the tests with sanitizers.

# Generating the library and the header that will be used during the test is
# necessary, before running `test_build_ffi` with sanitizers. If no FFI library
# is built, `cargo build --lib` will be executed in `test_build_ffi` to
# generate the FFI library we need. However, when `RUSTFLAGS="-Z sanitizer=*"`
# is set when building the FFI library, then the library will need symbols
# in `__*san_` pattern and we will need to set those `*San` symbol via `-l`
# when running `test_build_ffi`. Otherwise the linking will fail.
# To avoid those linking settings, the simplest way is to build the FFI
# library in advance, and running test with pre-built FFI library.
echo "Build C API library and header\n------------------------------"
cargo build --lib --features=capi

# Disable the build script. Avoding building crate with cbindgen when
# enabling sanitizers
sed '/package/a\
build = false
' $TEST_MANIFEST > temp.toml

mv temp.toml $TEST_MANIFEST

sanitizers=("address" "leak" "memory" "thread")
for san in "${sanitizers[@]}"
do
  San="$(tr '[:lower:]' '[:upper:]' <<< ${san:0:1})${san:1}"
  echo "\n\nRun ${San}Sanitizer\n------------------------------"
  if [ $san = "leak" ]; then
    echo "Skip the test that panics. Leaking memory when the program drops out abnormally is ok."
    options="-- --Z unstable-options --exclude-should-panic"
  fi
  RUSTFLAGS="-Z sanitizer=${san}" cargo test --features=capi $options
done
# -----------------------------------------------------------------------------

mv $ORIGINAL_MANIFEST $TEST_MANIFEST
