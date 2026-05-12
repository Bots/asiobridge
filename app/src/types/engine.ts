export interface ChannelState {
  id: number
  name: string
  active: boolean
  level: number
}

export interface RackState {
  id: string
  name: string
  channels: ChannelState[]
}

export interface ConnectionState {
  source_rack: string
  source_channel: number
  dest_rack: string
  dest_channel: number
  connection_type: string
  is_active: boolean
}

export interface EngineConfig {
  sample_rate: number
  bit_depth: number
  channels: number
}
