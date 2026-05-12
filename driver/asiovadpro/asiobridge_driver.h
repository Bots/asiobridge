// AsioBridge User-Mode Driver Interface
// Provides API to communicate with the kernel driver

#pragma once

#include <windows.h>
#include <iostream>

// IOCTL codes (must match kernel driver)
#define IOCTL_ASIOBRIDGE_GET_INFO \
  CTL_CODE(FILE_DEVICE_UNKNOWN, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_ASIOBRIDGE_SET_FORMAT \
  CTL_CODE(FILE_DEVICE_UNKNOWN, 0x801, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_ASIOBRIDGE_START \
  CTL_CODE(FILE_DEVICE_UNKNOWN, 0x802, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_ASIOBRIDGE_STOP \
  CTL_CODE(FILE_DEVICE_UNKNOWN, 0x803, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_ASIOBRIDGE_PAUSE \
  CTL_CODE(FILE_DEVICE_UNKNOWN, 0x804, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_ASIOBRIDGE_WRITE_AUDIO \
  CTL_CODE(FILE_DEVICE_UNKNOWN, 0x805, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define IOCTL_ASIOBRIDGE_READ_AUDIO \
  CTL_CODE(FILE_DEVICE_UNKNOWN, 0x806, METHOD_BUFFERED, FILE_ANY_ACCESS)

// IOCTL data structures
struct DeviceInfo {
  ULONG sampleRate;
  ULONG bitsPerSample;
  ULONG channelCount;
  ULONG bufferSize;
};

struct AudioFormat {
  ULONG sampleRate;
  ULONG bitsPerSample;
  ULONG channelCount;
};

class AsioBridgeDriver {
public:
  AsioBridgeDriver() : handle_(INVALID_HANDLE_VALUE) {}
  ~AsioBridgeDriver() { Close(); }

  bool Open() {
    handle_ = CreateFileW(
      L"\\\\.\\AsioBridgeVAD",
      GENERIC_READ | GENERIC_WRITE,
      0,
      nullptr,
      OPEN_EXISTING,
      FILE_ATTRIBUTE_NORMAL,
      nullptr
    );
    return handle_ != INVALID_HANDLE_VALUE;
  }

  void Close() {
    if (handle_ != INVALID_HANDLE_VALUE) {
      CloseHandle(handle_);
      handle_ = INVALID_HANDLE_VALUE;
    }
  }

  bool IsOpen() const {
    return handle_ != INVALID_HANDLE_VALUE;
  }

  bool GetDeviceInfo(DeviceInfo& info) {
    DWORD bytesReturned = 0;
    return DeviceIoControl(
      handle_,
      IOCTL_ASIOBRIDGE_GET_INFO,
      nullptr,
      0,
      &info,
      sizeof(info),
      &bytesReturned,
      nullptr
    );
  }

  bool SetFormat(const AudioFormat& format) {
    DWORD bytesReturned = 0;
    return DeviceIoControl(
      handle_,
      IOCTL_ASIOBRIDGE_SET_FORMAT,
      const_cast<AudioFormat*>(&format),
      sizeof(format),
      nullptr,
      0,
      &bytesReturned,
      nullptr
    );
  }

  bool Start() {
    DWORD bytesReturned = 0;
    return DeviceIoControl(
      handle_,
      IOCTL_ASIOBRIDGE_START,
      nullptr,
      0,
      nullptr,
      0,
      &bytesReturned,
      nullptr
    );
  }

  bool Stop() {
    DWORD bytesReturned = 0;
    return DeviceIoControl(
      handle_,
      IOCTL_ASIOBRIDGE_STOP,
      nullptr,
      0,
      nullptr,
      0,
      &bytesReturned,
      nullptr
    );
  }

  bool Pause(bool toggle) {
    DWORD bytesReturned = 0;
    return DeviceIoControl(
      handle_,
      IOCTL_ASIOBRIDGE_PAUSE,
      nullptr,
      0,
      nullptr,
      0,
      &bytesReturned,
      nullptr
    );
  }

  bool WriteAudio(const void* data, size_t length) {
    DWORD bytesReturned = 0;
    return DeviceIoControl(
      handle_,
      IOCTL_ASIOBRIDGE_WRITE_AUDIO,
      const_cast<void*>(data),
      static_cast<ULONG>(length),
      nullptr,
      0,
      &bytesReturned,
      nullptr
    );
  }

  bool ReadAudio(void* data, size_t length, size_t& bytesRead) {
    DWORD bytesReturned = 0;
    bool result = DeviceIoControl(
      handle_,
      IOCTL_ASIOBRIDGE_READ_AUDIO,
      nullptr,
      0,
      data,
      static_cast<ULONG>(length),
      &bytesReturned,
      nullptr
    );
    bytesRead = bytesReturned;
    return result;
  }

private:
  HANDLE handle_;
};
