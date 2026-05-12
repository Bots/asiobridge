//! Stub ASIO wrapper header — to be replaced with real implementation
//! when Steinberg ASIO SDK is available in vendor/

#pragma once

#ifdef _WIN32
#define ASIO_EXPORT __declspec(dllexport)
#else
#define ASIO_EXPORT
#endif

// Forward declarations for ASIO system calls
// Full implementation requires Steinberg ASIO SDK

extern "C" {
  ASIO_EXPORT int asioInit();
  ASIO_EXPORT int asioExit();
  ASIO_EXPORT int asioMessage(int selector, int value, void* ptr, float opt);
}
