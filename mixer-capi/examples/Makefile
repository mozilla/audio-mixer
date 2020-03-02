# Build C client example
# Process with GNU make

all: test

check: all
	./test

HEADER := ../include/audio_mixer.h
CXXFLAGS = -g -Wall -std=c++11 -I$(dir $(HEADER))

CRATE_DIR := $(OUT_DIR)/../../../deps

libaudio_mixer_capi.a libaudio_mixer_capi.a.out : ../src/lib.rs
	rustc -g --crate-type staticlib --crate-name audio_mixer_capi \
	  --emit dep-info,link=$@ \
	  -L $(CRATE_DIR) $< \
	  2> libaudio_mixer_capi.a.out || cat libaudio_mixer_capi.a.out >&2

-include audio_mixer_capi.d

test: RUST_LIBS = $(shell awk '/^note: library: / {print "-l"$$3}' libaudio_mixer_capi.a.out)
test: audio_mixer.c libaudio_mixer_capi.a $(HEADER)
	$(CXX) $(CXXFLAGS) -c $(filter %.c,$^)
	$(CXX) $(CXXFLAGS) -o $@ *.o libaudio_mixer_capi.a $(RUST_LIBS)

clean:
	$(RM) test
	$(RM) *.a.out
	$(RM) *.o *.a
	$(RM) *.d
