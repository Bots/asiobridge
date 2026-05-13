import { invoke } from '@tauri-apps/api/core'
import type { RackState, ConnectionState, EngineConfig, EngineStatus } from '@/types/engine'

export function useEngine() {
  const getRacks = async (): Promise<RackState[]> => {
    return invoke<RackState[]>('get_racks')
  }

  const getConnections = async (): Promise<ConnectionState[]> => {
    return invoke<ConnectionState[]>('get_connections')
  }

  const startEngine = async (): Promise<boolean> => {
    return invoke<boolean>('start_engine')
  }

  const stopEngine = async (): Promise<boolean> => {
    return invoke<boolean>('stop_engine')
  }

  const getEngineConfig = async (): Promise<EngineConfig> => {
    return invoke<EngineConfig>('get_engine_config')
  }

  const setSampleRate = async (sampleRate: number): Promise<number> => {
    return invoke<number>('set_sample_rate', { sampleRate })
  }

  const saveProfile = async (slot: number, name: string): Promise<boolean> => {
    return invoke<boolean>('save_profile', { slot, name })
  }

  const loadProfile = async (slot: number): Promise<boolean> => {
    return invoke<boolean>('load_profile', { slot })
  }

  const addConnection = async (
    sourceRack: string,
    sourceChannel: number,
    destRack: string,
    destChannel: number,
    connectionType: string
  ): Promise<boolean> => {
    return invoke<boolean>('add_connection', {
      sourceRack,
      sourceChannel,
      destRack,
      destChannel,
      connectionType,
    })
  }

  const removeConnection = async (
    sourceRack: string,
    sourceChannel: number
  ): Promise<boolean> => {
    return invoke<boolean>('remove_connection', {
      sourceRack,
      sourceChannel,
    })
  }

  const toggleConnection = async (
    sourceRack: string,
    sourceChannel: number,
    destRack: string,
    destChannel: number
  ): Promise<boolean> => {
    return invoke<boolean>('toggle_connection', {
      sourceRack,
      sourceChannel,
      destRack,
      destChannel,
    })
  }

  const setBitDepth = async (bitDepth: number): Promise<number> => {
    return invoke<number>('set_bit_depth', { bitDepth })
  }

  const getInputDevices = async (): Promise<string[]> => {
    return invoke<string[]>('get_input_devices')
  }

  const getOutputDevices = async (): Promise<string[]> => {
    return invoke<string[]>('get_output_devices')
  }

  const getDefaultInput = async (): Promise<string | null> => {
    return invoke<string | null>('get_default_input')
  }

  const getDefaultOutput = async (): Promise<string | null> => {
    return invoke<string | null>('get_default_output')
  }

  const startInputDevice = async (deviceName: string): Promise<string> => {
    return invoke<string>('start_input_device', { deviceName })
  }

  const startOutputDevice = async (deviceName: string): Promise<string> => {
    return invoke<string>('start_output_device', { deviceName })
  }

  const stopAudioDevice = async (): Promise<string> => {
    return invoke<string>('stop_audio_device')
  }

  const getEngineStatus = async (): Promise<EngineStatus> => {
    return invoke<EngineStatus>('get_engine_status')
  }

  const startNetworkStream = async (host: string, port: number): Promise<boolean> => {
    return invoke<boolean>('start_network_stream', { host, port })
  }

  const stopNetworkStream = async (): Promise<boolean> => {
    return invoke<boolean>('stop_network_stream')
  }

  const getNetworkStreamConfig = async (): Promise<{ host: string; port: number; is_active: boolean }> => {
    return invoke<{ host: string; port: number; is_active: boolean }>('get_network_stream_config')
  }

  const startRecording = async (outputDir: string): Promise<boolean> => {
    return invoke<boolean>('start_recording', { outputDir })
  }

  const stopRecording = async (): Promise<boolean> => {
    return invoke<boolean>('stop_recording')
  }

  const getRecordingStatus = async (): Promise<{ is_recording: boolean; output_dir: string }> => {
    return invoke<{ is_recording: boolean; output_dir: string }>('get_recording_status')
  }

  return {
    getRacks,
    getConnections,
    startEngine,
    stopEngine,
    getEngineConfig,
    getEngineStatus,
    startNetworkStream,
    stopNetworkStream,
    getNetworkStreamConfig,
    startRecording,
    stopRecording,
    getRecordingStatus,
    setSampleRate,
    setBitDepth,
    saveProfile,
    loadProfile,
    addConnection,
    removeConnection,
    toggleConnection,
    getInputDevices,
    getOutputDevices,
    getDefaultInput,
    getDefaultOutput,
    startInputDevice,
    startOutputDevice,
    stopAudioDevice,
  }
}
