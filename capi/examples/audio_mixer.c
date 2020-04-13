#include "audio_mixer.h"

#include <stdio.h>

#define ARRAY_ELEMS(a) (sizeof(a) / sizeof((a)[0]))

int main() {
  // The input and output channels are mapped to the test in lib.rs so it's
  // easier to check the result.
  const Channel input_channels[] = {CHANNEL_FRONT_LEFT,    CHANNEL_SILENCE,
                                    CHANNEL_FRONT_RIGHT,   CHANNEL_FRONT_CENTER,
                                    CHANNEL_BACK_LEFT,     CHANNEL_SIDE_RIGHT,
                                    CHANNEL_LOW_FREQUENCY, CHANNEL_SIDE_LEFT,
                                    CHANNEL_BACK_CENTER,   CHANNEL_BACK_RIGHT};
  const Channel output_channels[] = {CHANNEL_SILENCE, CHANNEL_FRONT_RIGHT,
                                     CHANNEL_FRONT_LEFT};

  Channels input_layout = {(const Channel*)&input_channels,
                           (uintptr_t)ARRAY_ELEMS(input_channels)};
  Channels output_layout = {(const Channel*)&output_channels,
                            (uintptr_t)ARRAY_ELEMS(output_channels)};

  float input_buffers[ARRAY_ELEMS(input_channels)] = {1, 2, 3, 4, 5,
                                                      6, 7, 8, 9, 10};

  float output_buffers[ARRAY_ELEMS(output_channels)] = {0};

  Handle handle = mixer_create(FORMAT_F32, input_layout, output_layout);

  mixer_mix(handle, &input_buffers, (uintptr_t)ARRAY_ELEMS(input_channels),
            &output_buffers, (uintptr_t)ARRAY_ELEMS(output_channels));

  for (size_t i = 0; i < ARRAY_ELEMS(input_buffers); i++) {
    printf("ch[%zu] = %f\n", i, input_buffers[i]);
  }
  printf("is mixed to\n");
  for (size_t i = 0; i < ARRAY_ELEMS(output_buffers); i++) {
    printf("ch[%zu] = %f\n", i, output_buffers[i]);
  }

  mixer_destroy(handle);

  return 0;
}
