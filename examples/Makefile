# Build C++ client example
# Process with GNU make

all: test

check: all
	./test

HEADER := ../include/audio_mixer.h
CXXFLAGS = -g -Wall -std=c++14 -I$(dir $(HEADER))

CRATE_DIR := $(OUT_DIR)/../../../deps

libaudiomixer.a libaudiomixer.a.out : ../src/lib.rs
	rustc -g --crate-type staticlib --crate-name audiomixer \
	  --emit dep-info,link=$@ --cfg 'feature="capi"' \
	  -L $(CRATE_DIR) $< \
	  2> libaudiomixer.a.out || cat libaudiomixer.a.out >&2

-include audiomixer.d

test: RUST_LIBS = $(shell awk '/^note: library: / {print "-l"$$3}' libaudiomixer.a.out)
test: audio_mixer.c libaudiomixer.a $(HEADER)
	$(CXX) $(CXXFLAGS) -c $(filter %.c,$^)
	$(CXX) $(CXXFLAGS) -o $@ *.o libaudiomixer.a $(RUST_LIBS)

clean:
	$(RM) test
	$(RM) *.a.out
	$(RM) *.o *.a
	$(RM) *.d