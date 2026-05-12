// Stub ASIO wrapper implementation
// Full implementation requires Steinberg ASIO SDK

#include "asio_wrapper.h"

extern "C" {
  ASIO_EXPORT int asioInit() {
    return 1;
  }

  ASIO_EXPORT int asioExit() {
    return 1;
  }

  ASIO_EXPORT int asioMessage(int selector, int value, void* ptr, float opt) {
    return 0;
  }
}
