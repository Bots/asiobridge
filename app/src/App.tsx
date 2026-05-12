import React, { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import { useEngine } from '@/hooks/useEngine'
import type { RackState, ConnectionState, EngineConfig, EngineStatus } from '@/types/engine'

const SAMPLE_RATES = [44100, 48000, 88200, 96000, 176400, 192000]
const BIT_DEPTHS = [16, 24, 32]

const RACK_COLORS: Record<string, string> = {
  'asio-driver-in': 'border-blue-500',
  'asio-driver-out': 'border-green-500',
  'asio-host-in': 'border-purple-500',
  'network-in': 'border-orange-500',
  'network-out': 'border-orange-500',
  'looper-in': 'border-yellow-500',
  'looper-out': 'border-yellow-500',
  'wdm-in': 'border-red-500',
  'mix-out': 'border-cyan-500',
}

function ChannelStrip({ channel }: { channel: RackState['channels'][0] }) {
  return (
    <div className="flex flex-col items-center gap-2">
      <span className="text-xs text-muted-foreground">{channel.name}</span>
      <div className="flex h-48 flex-col items-center gap-1">
        <Slider
          value={[channel.level]}
          max={1}
          step={0.01}
          className="h-48 rotate-180"
        />
        <span className="text-xs text-muted-foreground">{Math.round(channel.level * 100)}%</span>
      </div>
      <Switch checked={channel.active} />
    </div>
  )
}

function RackView({ rack }: { rack: RackState }) {
  const colorClass = RACK_COLORS[rack.id] || 'border-gray-500'

  return (
    <Card className={cn('w-48 shrink-0', colorClass)}>
      <CardHeader className="p-3">
        <CardTitle className="text-xs">{rack.name}</CardTitle>
      </CardHeader>
      <CardContent className="flex justify-between p-3">
        {rack.channels.map((ch) => (
          <ChannelStrip key={ch.id} channel={ch} />
        ))}
      </CardContent>
    </Card>
  )
}

function ConnectionMatrix({ connections }: { connections: ConnectionState[] }) {
  const rackIds = [
    'asio-driver-in',
    'asio-driver-out',
    'asio-host-in',
    'network-in',
    'network-out',
    'mix-out',
  ]
  const rackNames = rackIds.map((id) => id.split('-').map((w) => w[0].toUpperCase() + w.slice(1)).join(' '))

  return (
    <Card>
      <CardHeader className="p-3">
        <CardTitle className="text-sm">Connection Matrix</CardTitle>
      </CardHeader>
      <CardContent className="p-3">
        <div className="grid gap-1" style={{ gridTemplateColumns: `auto repeat(${rackIds.length}, auto)` }}>
          <div></div>
          {rackNames.map((name, i) => (
            <div key={i} className="text-center text-xs text-muted-foreground">{name}</div>
          ))}
          {rackIds.map((srcId, srcIdx) => (
            <React.Fragment key={srcId}>
              <div className="text-center text-xs text-muted-foreground">{rackNames[srcIdx]}</div>
              {rackIds.map((dstId) => {
                const conn = connections.find(
                  (c) => c.source_rack === srcId && c.dest_rack === dstId
                )
                const isActive = conn?.is_active ?? false
                return (
                  <div key={dstId} className="h-8 w-8">
                    <Button
                      variant={isActive ? 'default' : 'outline'}
                      size="sm"
                      className="h-full w-full text-[8px]"
                    >
                      {srcId === dstId ? 'X' : '→'}
                    </Button>
                  </div>
                )
              })}
            </React.Fragment>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}

function DevicePanel() {
  const {
    getInputDevices,
    getOutputDevices,
    getDefaultInput,
    getDefaultOutput,
    startInputDevice,
    startOutputDevice,
    stopAudioDevice,
    startNetworkStream,
    stopNetworkStream,
    getNetworkStreamConfig,
  } = useEngine()

  const [inputDevices, setInputDevices] = useState<string[]>([])
  const [outputDevices, setOutputDevices] = useState<string[]>([])
  const [selectedInput, setSelectedInput] = useState<string>('')
  const [selectedOutput, setSelectedOutput] = useState<string>('')
  const [networkHost, setNetworkHost] = useState('127.0.0.1')
  const [networkPort, setNetworkPort] = useState(6997)
  const [isAudioRunning, setIsAudioRunning] = useState(false)
  const [isNetworkStreaming, setIsNetworkStreaming] = useState(false)

  useEffect(() => {
    Promise.all([getInputDevices(), getOutputDevices()]).then(
      ([input, output]) => {
        setInputDevices(input)
        setOutputDevices(output)
        if (input.length > 0) {
          getDefaultInput().then((d) => {
            if (d) setSelectedInput(d)
          })
        }
        if (output.length > 0) {
          getDefaultOutput().then((d) => {
            if (d) setSelectedOutput(d)
          })
        }
      }
    )

    getNetworkStreamConfig().then((config) => {
      setNetworkHost(config.host)
      setNetworkPort(config.port)
      setIsNetworkStreaming(config.is_active)
    })
  }, [])

  const handleStartAudio = async () => {
    try {
      if (selectedInput) {
        await startInputDevice(selectedInput)
      }
      if (selectedOutput) {
        await startOutputDevice(selectedOutput)
      }
      setIsAudioRunning(true)
    } catch (e) {
      console.error('Failed to start audio:', e)
    }
  }

  const handleStopAudio = async () => {
    try {
      await stopAudioDevice()
      setIsAudioRunning(false)
    } catch (e) {
      console.error('Failed to stop audio:', e)
    }
  }

  const handleStartNetworkStream = async () => {
    try {
      await startNetworkStream(networkHost, networkPort)
      setIsNetworkStreaming(true)
    } catch (e) {
      console.error('Failed to start network stream:', e)
    }
  }

  const handleStopNetworkStream = async () => {
    try {
      await stopNetworkStream()
      setIsNetworkStreaming(false)
    } catch (e) {
      console.error('Failed to stop network stream:', e)
    }
  }

  return (
    <Card>
      <CardHeader className="p-3">
        <CardTitle className="text-sm">Device Configuration</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4 p-3">
        <div className="space-y-2">
          <Label>Input Device</Label>
          <Select value={selectedInput} onValueChange={setSelectedInput}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select input device" />
            </SelectTrigger>
            <SelectContent>
              {inputDevices.map((device) => (
                <SelectItem key={device} value={device}>
                  {device}
                </SelectItem>
              ))}
              {inputDevices.length === 0 && (
                <SelectItem value="none" disabled>
                  No input devices found
                </SelectItem>
              )}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <Label>Output Device</Label>
          <Select value={selectedOutput} onValueChange={setSelectedOutput}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select output device" />
            </SelectTrigger>
            <SelectContent>
              {outputDevices.map((device) => (
                <SelectItem key={device} value={device}>
                  {device}
                </SelectItem>
              ))}
              {outputDevices.length === 0 && (
                <SelectItem value="none" disabled>
                  No output devices found
                </SelectItem>
              )}
            </SelectContent>
          </Select>
        </div>

<div className="space-y-2">
            <Label>Network Streaming</Label>
            <div className="flex gap-2">
              <Input
                placeholder="127.0.0.1"
                value={networkHost}
                onChange={(e) => setNetworkHost(e.target.value)}
              />
              <Input
                type="number"
                placeholder="6997"
                value={networkPort}
                onChange={(e) => setNetworkPort(Number(e.target.value))}
                className="w-20"
              />
            </div>
            <div className="flex gap-2">
              <Button
                size="sm"
                onClick={handleStartNetworkStream}
                disabled={isNetworkStreaming}
              >
                Start Stream
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleStopNetworkStream}
                disabled={!isNetworkStreaming}
              >
                Stop Stream
              </Button>
            </div>
          </div>

        <div className="flex gap-2">
          <Button
            size="sm"
            onClick={handleStartAudio}
            disabled={isAudioRunning || inputDevices.length === 0 || outputDevices.length === 0}
          >
            Start Audio
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={handleStopAudio}
            disabled={!isAudioRunning}
          >
            Stop Audio
          </Button>
        </div>

        <div className="flex items-center gap-2">
          <Badge variant="outline">cpal backend</Badge>
          <Badge variant="outline">ASIO (Windows)</Badge>
        </div>
      </CardContent>
    </Card>
  )
}

function App() {
  const {
    getRacks,
    getConnections,
    getEngineConfig,
    getEngineStatus,
    startEngine,
    stopEngine,
    setSampleRate,
    saveProfile,
    loadProfile,
  } = useEngine()

  const [racks, setRacks] = useState<RackState[]>([])
  const [connections, setConnections] = useState<ConnectionState[]>([])
  const [config, setConfig] = useState<EngineConfig | null>(null)
  const [status, setStatus] = useState<EngineStatus | null>(null)
  const [selectedProfile, setSelectedProfile] = useState(0)

  useEffect(() => {
    Promise.all([getRacks(), getConnections(), getEngineConfig(), getEngineStatus()]).then(
      ([racksData, connsData, configData, statusData]) => {
        setRacks(racksData)
        setConnections(connsData)
        setConfig(configData)
        setStatus(statusData)
      }
    )

    const interval = setInterval(async () => {
      const statusData = await getEngineStatus()
      setStatus(statusData)
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  const handleStartStop = async () => {
    if (status?.is_running) {
      await stopEngine()
    } else {
      await startEngine()
    }
  }

  const handleSampleRateChange = async (rate: number) => {
    await setSampleRate(rate)
    const configData = await getEngineConfig()
    setConfig(configData)
  }

  const handleSaveProfile = async () => {
    await saveProfile(selectedProfile, `Profile ${selectedProfile + 1}`)
  }

  const handleLoadProfile = async () => {
    await loadProfile(selectedProfile)
    const racksData = await getRacks()
    setRacks(racksData)
  }

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b bg-card px-4 py-3">
        <div className="flex items-center justify-between">
          <h1 className="text-lg font-bold">AsioBridge</h1>
          <div className="flex items-center gap-2">
            <select
              className="rounded border bg-background px-2 py-1 text-sm"
              value={selectedProfile}
              onChange={(e) => setSelectedProfile(Number(e.target.value))}
            >
              {Array.from({ length: 8 }, (_, i) => (
                <option key={i} value={i}>Profile {i + 1}</option>
              ))}
            </select>
            <Button variant="outline" size="sm" onClick={handleSaveProfile}>Save</Button>
            <Button variant="outline" size="sm" onClick={handleLoadProfile}>Load</Button>
          </div>
        </div>
      </header>

      <main className="p-4">
        <div className="mb-4 flex gap-2">
          <Select>
            <SelectTrigger className="w-64">
              <SelectValue placeholder="Select ASIO driver" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="asio">ASIO: AsioBridge Virtual Driver</SelectItem>
            </SelectContent>
          </Select>
          <Select
            value={config?.sample_rate?.toString() ?? '44100'}
            onValueChange={(v: string) => handleSampleRateChange(Number(v))}
          >
            <SelectTrigger className="w-32">
              <SelectValue placeholder="Sample rate" />
            </SelectTrigger>
            <SelectContent>
              {SAMPLE_RATES.map((rate) => (
                <SelectItem key={rate} value={rate.toString()}>{rate} Hz</SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select>
            <SelectTrigger className="w-24">
              <SelectValue placeholder="Bit depth" />
            </SelectTrigger>
            <SelectContent>
              {BIT_DEPTHS.map((bits) => (
                <SelectItem key={bits} value={bits.toString()}>{bits} bit</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="mb-4 flex gap-4 overflow-x-auto pb-4">
          {racks.map((rack) => (
            <RackView key={rack.id} rack={rack} />
          ))}
        </div>

        <div className="mb-4 grid grid-cols-2 gap-4">
          <DevicePanel />
        </div>

        <ConnectionMatrix connections={connections} />
      </main>

      <footer className="border-t bg-card px-4 py-2">
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-4">
            <span className="text-muted-foreground">
              Sample Rate: {status?.sample_rate ?? config?.sample_rate ?? '---'} Hz
            </span>
            <span className="text-muted-foreground">
              Bit Depth: {status?.bit_depth ?? config?.bit_depth ?? '---'} bit
            </span>
            <span className="text-muted-foreground">
              Channels: {status?.channels ?? config?.channels ?? '---'}
            </span>
            <span className={status?.is_running ? 'text-green-500' : 'text-red-500'}>
              {status?.is_running ? '● Running' : '○ Stopped'}
            </span>
          </div>
          <Button size="sm" onClick={handleStartStop} variant={status?.is_running ? 'destructive' : 'default'}>
            {status?.is_running ? 'Stop' : 'Start'}
          </Button>
        </div>
      </footer>
    </div>
  )
}

export default App
