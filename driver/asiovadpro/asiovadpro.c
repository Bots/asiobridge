// AsioBridge Virtual Audio Driver (asiovadpro)
// WDM kernel driver for Windows
// This driver creates a virtual audio endpoint that applications can use as an ASIO device

#include <ntddk.h>
#include <wdm.h>
#include <usbiodef.h>
#include <mountdev.h>

#define DRIVER_NAME "AsioBridge Virtual Audio Driver"
#define DEVICE_NAME L"\\Device\\AsioBridgeVAD"
#define DOS_DEVICE_NAME L"\\DosDevices\\AsioBridgeVAD"

// Audio stream format constants
#define SAMPLE_RATE_44100 44100
#define SAMPLE_RATE_48000 48000
#define BITS_PER_SAMPLE 24
#define CHANNEL_COUNT 8

// Shared memory for audio data exchange between kernel and user mode
#define SHARED_MEMORY_SIZE (8 * 1024) // 8KB ring buffer

// Device extension for our virtual audio device
typedef struct _DEVICE_EXTENSION {
  PDEVICE_OBJECT fdo;
  UNICODE_STRING deviceName;
  UNICODE_STRING dosDeviceName;
  KSPIN_LOCK lock;
  BOOLEAN started;
  BOOLEAN paused;
  ULONG sampleRate;
  ULONG bitsPerSample;
  ULONG channelCount;
  PVOID sharedBuffer;
  ULONG bufferWritePos;
  ULONG bufferReadPos;
  ULONG bufferUsed;
} DEVICE_EXTENSION, *PDEVICE_EXTENSION;

// Forward declarations
VOID DriverUnload(PDRIVER_OBJECT DriverObject);
NTSTATUS CreateHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp);
NTSTATUS CloseHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp);
NTSTATUS IoControlHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp);
NTSTATUS CleanupHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp);

NTSTATUS
DriverEntry(
  PDRIVER_OBJECT DriverObject,
  PUNICODE_STRING RegistryPath
)
{
  NTSTATUS status;
  PDEVICE_OBJECT deviceObj;
  UNICODE_STRING uniDeviceName;
  UNICODE_STRING uniDosDeviceName;

  UNREFERENCED_PARAMETER(RegistryPath);

  // Initialize driver object callbacks
  for (ULONG i = 0; i < IRP_MJ_MAXIMUM_FUNCTION; i++) {
    DriverObject->MajorFunction[i] = IoDefaultDeviceHandler;
  }

  DriverObject->MajorFunction[IRP_MJ_CREATE] = CreateHandler;
  DriverObject->MajorFunction[IRP_MJ_CLOSE] = CloseHandler;
  DriverObject->MajorFunction[IRP_MJ_CLEANUP] = CleanupHandler;
  DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = IoControlHandler;
  DriverObject->DriverUnload = DriverUnload;

  // Create device name
  RtlInitUnicodeString(&uniDeviceName, DEVICE_NAME);
  RtlInitUnicodeString(&uniDosDeviceName, DOS_DEVICE_NAME);

  // Create the kernel device
  status = IoCreateDevice(
    DriverObject,
    sizeof(DEVICE_EXTENSION),
    &uniDeviceName,
    FILE_DEVICE_UNKNOWN,
    FILE_DEVICE_SECURE_OPEN,
    FALSE,
    &deviceObj
  );

  if (!NT_SUCCESS(status)) {
    return status;
  }

  // Initialize device extension
  PDEVICE_EXTENSION devExt = (PDEVICE_EXTENSION)deviceObj->DeviceExtension;
  devExt->fdo = deviceObj;
  devExt->deviceName = uniDeviceName;
  devExt->dosDeviceName = uniDosDeviceName;
  KeInitializeSpinLock(&devExt->lock);
  devExt->started = FALSE;
  devExt->paused = FALSE;
  devExt->sampleRate = SAMPLE_RATE_44100;
  devExt->bitsPerSample = BITS_PER_SAMPLE;
  devExt->channelCount = CHANNEL_COUNT;
  devExt->sharedBuffer = ExAllocatePool2(
    POOL_FLAG_NON_PAGED,
    SHARED_MEMORY_SIZE,
    'BSiA'
  );
  devExt->bufferWritePos = 0;
  devExt->bufferReadPos = 0;
  devExt->bufferUsed = 0;

  // Create DOS device link
  status = IoCreateSymbolicLink(&uniDosDeviceName, &uniDeviceName);
  if (!NT_SUCCESS(status)) {
    if (devExt->sharedBuffer) {
      ExFreePool2(devExt->sharedBuffer, 0);
    }
    IoDeleteDevice(deviceObj);
    return status;
  }

  deviceObj->Flags |= DO_BUFFERED_IO;
  deviceObj->Flags &= ~DO_DEVICE_INITIALIZING;

  return STATUS_SUCCESS;
}

VOID
DriverUnload(PDRIVER_OBJECT DriverObject)
{
  PDEVICE_OBJECT deviceObj = DriverObject->DeviceObject;
  UNICODE_STRING uniDosDeviceName;

  while (deviceObj) {
    PDEVICE_EXTENSION devExt = (PDEVICE_EXTENSION)deviceObj->DeviceExtension;

    if (devExt->sharedBuffer) {
      ExFreePool2(devExt->sharedBuffer, 0);
    }

    RtlInitUnicodeString(&uniDosDeviceName, DOS_DEVICE_NAME);
    IoDeleteSymbolicLink(&uniDosDeviceName);
    IoDeleteDevice(deviceObj);

    deviceObj = deviceObj->NextDevice;
  }
}

NTSTATUS
CreateHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
  PDEVICE_EXTENSION devExt = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;

  // Zero the shared buffer on first open
  if (!devExt->started) {
    RtlZeroMemory(devExt->sharedBuffer, SHARED_MEMORY_SIZE);
    devExt->bufferWritePos = 0;
    devExt->bufferReadPos = 0;
    devExt->bufferUsed = 0;
  }

  Irp->IoStatus.Status = STATUS_SUCCESS;
  Irp->IoStatus.Information = 0;
  IoCompleteRequest(Irp, IO_NO_INCREMENT);

  return STATUS_SUCCESS;
}

NTSTATUS
CloseHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
  UNREFERENCED_PARAMETER(DeviceObject);

  Irp->IoStatus.Status = STATUS_SUCCESS;
  Irp->IoStatus.Information = 0;
  IoCompleteRequest(Irp, IO_NO_INCREMENT);

  return STATUS_SUCCESS;
}

NTSTATUS
CleanupHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
  UNREFERENCED_PARAMETER(DeviceObject);

  Irp->IoStatus.Status = STATUS_SUCCESS;
  Irp->IoStatus.Information = 0;
  IoCompleteRequest(Irp, IO_NO_INCREMENT);

  return STATUS_SUCCESS;
}

NTSTATUS
IoControlHandler(PDEVICE_OBJECT DeviceObject, PIRP Irp)
{
  PDEVICE_EXTENSION devExt = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
  PIO_STACK_LOCATION stack = IoGetCurrentIrpStackLocation(Irp);
  ULONG ioControlCode = stack->Parameters.DeviceIoControl.IoControlCode;
  PVOID inputBuffer = Irp->AssociatedIrp.SystemBuffer;
  ULONG inputBufferLength = stack->Parameters.DeviceIoControl.InputBufferLength;
  PVOID outputBuffer = Irp->AssociatedIrp.SystemBuffer;
  ULONG outputBufferLength = stack->Parameters.DeviceIoControl.OutputBufferLength;
  ULONG bytesReturned = 0;
  NTSTATUS status = STATUS_SUCCESS;

  switch (ioControlCode) {
    case IOCTL_ASIOBRIDGE_GET_INFO: {
      if (outputBufferLength < sizeof(DEVICE_INFO)) {
        status = STATUS_BUFFER_TOO_SMALL;
        break;
      }
      PDEVICE_INFO info = (PDEVICE_INFO)outputBuffer;
      info->sampleRate = devExt->sampleRate;
      info->bitsPerSample = devExt->bitsPerSample;
      info->channelCount = devExt->channelCount;
      info->bufferSize = SHARED_MEMORY_SIZE;
      bytesReturned = sizeof(DEVICE_INFO);
      break;
    }
    case IOCTL_ASIOBRIDGE_SET_FORMAT: {
      if (inputBufferLength < sizeof(AUDIO_FORMAT)) {
        status = STATUS_BUFFER_TOO_SMALL;
        break;
      }
      PAUDIO_FORMAT format = (PAUDIO_FORMAT)inputBuffer;
      devExt->sampleRate = format->sampleRate;
      devExt->bitsPerSample = format->bitsPerSample;
      devExt->channelCount = format->channelCount;
      break;
    }
    case IOCTL_ASIOBRIDGE_START: {
      devExt->started = TRUE;
      devExt->paused = FALSE;
      break;
    }
    case IOCTL_ASIOBRIDGE_STOP: {
      devExt->started = FALSE;
      devExt->paused = FALSE;
      break;
    }
    case IOCTL_ASIOBRIDGE_PAUSE: {
      devExt->paused = !devExt->paused;
      break;
    }
    case IOCTL_ASIOBRIDGE_WRITE_AUDIO: {
      if (inputBufferLength == 0) {
        status = STATUS_INVALID_BUFFER_SIZE;
        break;
      }
      KeAcquireSpinLock(&devExt->lock, NULL);
      ULONG bytesToWrite = min(inputBufferLength, SHARED_MEMORY_SIZE - devExt->bufferUsed);
      RtlCopyMemory(
        (PCHAR)devExt->sharedBuffer + devExt->bufferWritePos,
        inputBuffer,
        bytesToWrite
      );
      devExt->bufferWritePos = (devExt->bufferWritePos + bytesToWrite) % SHARED_MEMORY_SIZE;
      devExt->bufferUsed += bytesToWrite;
      KeReleaseSpinLock(&devExt->lock, DISPATCH_LEVEL);
      bytesReturned = bytesToWrite;
      break;
    }
    case IOCTL_ASIOBRIDGE_READ_AUDIO: {
      if (outputBufferLength == 0) {
        status = STATUS_INVALID_BUFFER_SIZE;
        break;
      }
      KeAcquireSpinLock(&devExt->lock, NULL);
      ULONG bytesToRead = min(outputBufferLength, devExt->bufferUsed);
      RtlCopyMemory(
        outputBuffer,
        (PCHAR)devExt->sharedBuffer + devExt->bufferReadPos,
        bytesToRead
      );
      devExt->bufferReadPos = (devExt->bufferReadPos + bytesToRead) % SHARED_MEMORY_SIZE;
      devExt->bufferUsed -= bytesToRead;
      KeReleaseSpinLock(&devExt->lock, DISPATCH_LEVEL);
      bytesReturned = bytesToRead;
      break;
    }
    default: {
      status = STATUS_INVALID_DEVICE_REQUEST;
      break;
    }
  }

  Irp->IoStatus.Status = status;
  Irp->IoStatus.Information = bytesReturned;
  IoCompleteRequest(Irp, IO_NO_INCREMENT);

  return status;
}

// Device control codes
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
typedef struct _DEVICE_INFO {
  ULONG sampleRate;
  ULONG bitsPerSample;
  ULONG channelCount;
  ULONG bufferSize;
} DEVICE_INFO, *PDEVICE_INFO;

typedef struct _AUDIO_FORMAT {
  ULONG sampleRate;
  ULONG bitsPerSample;
  ULONG channelCount;
} AUDIO_FORMAT, *PAUDIO_FORMAT;
