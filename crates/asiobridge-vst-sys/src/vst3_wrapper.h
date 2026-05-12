#pragma once

#ifdef _WIN32
#define VST3_EXPORT __declspec(dllexport)
#else
#define VST3_EXPORT
#endif

extern "C" {
  VST3_EXPORT int vst3Init();
  VST3_EXPORT int vst3Exit();
}
