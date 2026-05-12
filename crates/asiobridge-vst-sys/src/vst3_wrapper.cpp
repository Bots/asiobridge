#include "vst3_wrapper.h"

extern "C" {
  VST3_EXPORT int vst3Init() {
    return 1;
  }

  VST3_EXPORT int vst3Exit() {
    return 1;
  }
}
