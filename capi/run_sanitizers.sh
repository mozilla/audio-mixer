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

# `cbindgen` cannot work with sanitizers, so we need to avoid building crate
# with `cbindgen` when enabling sanitizers. We can generate the library and 
# the header that will be used during the test in advance and then disable 
# the build script.

# Generate the library and the header that will be used during the test
echo "Build C API library and header\n------------------------------"
cargo build --lib

# Disable the build script. Avoding building crate with cbindgen when
# enabling sanitizers
ORIGINAL_MANIFEST="Cargo_ori.toml"
TEST_MANIFEST="Cargo.toml"
mv $TEST_MANIFEST $ORIGINAL_MANIFEST
# Delete lines that contain a `criterion` from Cargo.toml.
sed '/package/a\
build = false
' $ORIGINAL_MANIFEST > $TEST_MANIFEST

sanitizers=("address" "leak" "memory" "thread")
for san in "${sanitizers[@]}"
do
  San="$(tr '[:lower:]' '[:upper:]' <<< ${san:0:1})${san:1}"
  echo "\n\nRun ${San}Sanitizer\n------------------------------"
  if [ $san = "leak" ]; then
    options=""
  else
    echo "Avoid running 'test_build_ffi' since we will have a 'Undefined symbols' for __*san_*"
    options="test_c_api"
  fi

  RUSTFLAGS="-Z sanitizer=${san}" cargo test $options
done

mv $ORIGINAL_MANIFEST $TEST_MANIFEST
