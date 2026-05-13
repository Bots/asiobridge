import { useEffect, useState } from 'react'
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
import { useToast, ToastProvider } from '@/components/ui/toast'
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts'
import { ConnectionPanel } from '@/components/ConnectionPanel'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
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

const LEVEL_COLORS = [
  'bg-green-500',
  'bg-green-400',
  'bg-lime-400',
  'bg-yellow-400',
  'bg-orange-400',
  'bg-red-500',
  'bg-red-600',
]

function LevelMeter({ level }: { level: number }) {
  const segments = 12
  const activeSegments = Math.floor(level * segments)

  return (
    <div className="flex h-32 w-3 flex-col-reverse gap-0.5 rounded-sm overflow-hidden bg-muted">
      {Array.from({ length: segments }).map((_, i) => {
        const isActive = i < activeSegments
        const colorIndex = Math.min(Math.floor((i / segments) * LEVEL_COLORS.length), LEVEL_COLORS.length - 1)
        return (
          <div
            key={i}
            className={cn(
              'flex-1 rounded-sm transition-all duration-75',
              isActive ? LEVEL_COLORS[colorIndex] : 'bg-muted-foreground/20'
            )}
          />
        )
      })}
    </div>
  )
}

function ChannelStrip({ channel }: { channel: RackState['channels'][0] }) {
  const [currentLevel, setCurrentLevel] = useState(0)

  useEffect(() => {
    if (!channel.active) {
      setCurrentLevel(0)
      return
    }
    const interval = setInterval(() => {
      setCurrentLevel(Math.random() * 0.3 * channel.level)
    }, 100)
    return () => clearInterval(interval)
  }, [channel.active, channel.level])

  return (
    <div className="flex flex-col items-center gap-1">
      <span className="text-[10px] text-muted-foreground">{channel.name}</span>
      <div className="flex items-center gap-1">
        <LevelMeter level={currentLevel} />
        <div className="flex flex-col items-center gap-1">
          <Slider
            value={[channel.level]}
            max={1}
            step={0.01}
            className="h-48 rotate-180"
          />
          <span className="text-[10px] text-muted-foreground">{Math.round(channel.level * 100)}%</span>
        </div>
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

function MixerView({ channels }: { channels: RackState['channels'] }) {
  return (
    <Card>
      <CardHeader className="p-3">
        <CardTitle className="text-sm">Mixer</CardTitle>
      </CardHeader>
      <CardContent className="p-3">
        <div className="flex items-end justify-between gap-2 overflow-x-auto pb-2">
          {channels.map((channel) => (
            <div key={channel.id} className="flex flex-col items-center gap-1">
              <span className="text-[10px] text-muted-foreground">{channel.name}</span>
              <div className="relative h-48 w-8 rounded-lg bg-muted p-1">
                <div className="absolute bottom-0 left-0 right-0 rounded-sm bg-gradient-to-t from-green-500 via-yellow-400 to-red-500 transition-all"
                  style={{ height: `${channel.level * 100}%` }}
                />
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={channel.level}
                  className="absolute inset-0 h-48 w-8 opacity-0 cursor-pointer"
                  style={{ writingMode: 'vertical-lr', direction: 'rtl' }}
                />
              </div>
              <span className="text-[10px] text-muted-foreground">{Math.round(channel.level * 100)}%</span>
              <Switch checked={channel.active} />
            </div>
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
    startRecording,
    stopRecording,
    getRecordingStatus,
  } = useEngine()
  const { addToast } = useToast()

  const [inputDevices, setInputDevices] = useState<string[]>([])
  const [outputDevices, setOutputDevices] = useState<string[]>([])
  const [selectedInput, setSelectedInput] = useState<string>('')
  const [selectedOutput, setSelectedOutput] = useState<string>('')
  const [networkHost, setNetworkHost] = useState('127.0.0.1')
  const [networkPort, setNetworkPort] = useState(6997)
  const [isAudioRunning, setIsAudioRunning] = useState(false)
  const [isNetworkStreaming, setIsNetworkStreaming] = useState(false)
  const [isRecording, setIsRecording] = useState(false)

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

    getRecordingStatus().then((status) => {
      setIsRecording(status.is_recording)
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
      addToast('Audio started', 'success')
    } catch (e) {
      console.error('Failed to start audio:', e)
      addToast('Failed to start audio', 'error')
    }
  }

  const handleStopAudio = async () => {
    try {
      await stopAudioDevice()
      setIsAudioRunning(false)
      addToast('Audio stopped', 'info')
    } catch (e) {
      console.error('Failed to stop audio:', e)
      addToast('Failed to stop audio', 'error')
    }
  }

  const handleStartNetworkStream = async () => {
    try {
      await startNetworkStream(networkHost, networkPort)
      setIsNetworkStreaming(true)
      addToast(`Network streaming to ${networkHost}:${networkPort}`, 'success')
    } catch (e) {
      console.error('Failed to start network stream:', e)
      addToast('Failed to start network stream', 'error')
    }
  }

  const handleStopNetworkStream = async () => {
    try {
      await stopNetworkStream()
      setIsNetworkStreaming(false)
      addToast('Network streaming stopped', 'info')
    } catch (e) {
      console.error('Failed to stop network stream:', e)
      addToast('Failed to stop network stream', 'error')
    }
  }

  const handleStartRecording = async () => {
    try {
      await startRecording('')
      setIsRecording(true)
      addToast('Recording started', 'success')
    } catch (e) {
      console.error('Failed to start recording:', e)
      addToast('Failed to start recording', 'error')
    }
  }

  const handleStopRecording = async () => {
    try {
      await stopRecording()
      setIsRecording(false)
      addToast('Recording stopped', 'info')
    } catch (e) {
      console.error('Failed to stop recording:', e)
      addToast('Failed to stop recording', 'error')
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
            <div className="flex gap-2 mt-2">
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

        <div className="space-y-2">
          <Label>Recording</Label>
          <div className="flex gap-2">
            <Button
              size="sm"
              onClick={isRecording ? handleStopRecording : handleStartRecording}
              disabled={false}
              variant={isRecording ? 'default' : 'destructive'}
            >
              {isRecording ? '⏹ Stop Rec' : '⏺ Start Rec'}
            </Button>
            <Badge variant={isRecording ? 'default' : 'outline'}>
              {isRecording ? '● Recording' : '○ Ready'}
            </Badge>
          </div>
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

  useKeyboardShortcuts([
    {
      key: 's',
      ctrl: true,
      action: handleSaveProfile,
    },
    {
      key: 'o',
      ctrl: true,
      action: handleLoadProfile,
    },
    {
      key: ' ',
      action: handleStartStop,
    },
  ])

  return (
    <div className="min-h-screen bg-background">
      <ToastProvider>
      <header className="border-b bg-card px-4 py-3">
        <div className="flex items-center justify-between">
          <h1 className="text-lg font-bold">AsioBridge</h1>
          <div className="flex items-center gap-2">
            <Select value={selectedProfile.toString()} onValueChange={(v) => setSelectedProfile(Number(v))}>
              <SelectTrigger className="w-32">
                <SelectValue placeholder="Profile" />
              </SelectTrigger>
              <SelectContent>
                {Array.from({ length: 8 }, (_, i) => (
                  <SelectItem key={i} value={i.toString()}>Profile {i + 1}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button variant="outline" size="sm" onClick={handleSaveProfile}>Save</Button>
            <Button variant="outline" size="sm" onClick={handleLoadProfile}>Load</Button>
          </div>
        </div>
      </header>

      <main className="p-4">
        <div className="mb-4 flex gap-2">
          <Select defaultValue="asio">
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
          <Select defaultValue="24">
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

        <Tabs defaultValue="racks">
          <TabsList className="mb-4">
            <TabsTrigger value="racks">Racks</TabsTrigger>
            <TabsTrigger value="connections">Connections</TabsTrigger>
            <TabsTrigger value="devices">Devices</TabsTrigger>
          </TabsList>

          <TabsContent value="racks" className="space-y-4">
            <div className="flex gap-4 overflow-x-auto pb-4">
              {racks.map((rack) => (
                <RackView key={rack.id} rack={rack} />
              ))}
            </div>

            {racks.find(r => r.id === 'mix-out') && (
              <MixerView channels={racks.find(r => r.id === 'mix-out')!.channels} />
            )}
          </TabsContent>

          <TabsContent value="connections">
            <ConnectionPanel connections={connections} />
          </TabsContent>

          <TabsContent value="devices">
            <DevicePanel />
          </TabsContent>
        </Tabs>
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
      </ToastProvider>
    </div>
  )
}

export default App
