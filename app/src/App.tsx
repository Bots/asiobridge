import React, { useEffect, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'
import { useEngine } from '@/hooks/useEngine'
import type { RackState, ConnectionState, EngineConfig } from '@/types/engine'

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

function App() {
  const {
    getRacks,
    getConnections,
    getEngineConfig,
    startEngine,
    stopEngine,
    setSampleRate,
    saveProfile,
    loadProfile,
  } = useEngine()

  const [racks, setRacks] = useState<RackState[]>([])
  const [connections, setConnections] = useState<ConnectionState[]>([])
  const [config, setConfig] = useState<EngineConfig | null>(null)
  const [isRunning, setIsRunning] = useState(false)
  const [selectedProfile, setSelectedProfile] = useState(0)

  useEffect(() => {
    Promise.all([getRacks(), getConnections(), getEngineConfig()]).then(
      ([racksData, connsData, configData]) => {
        setRacks(racksData)
        setConnections(connsData)
        setConfig(configData)
      }
    )
  }, [])

  const handleStartStop = async () => {
    if (isRunning) {
      const stopped = await stopEngine()
      setIsRunning(stopped)
    } else {
      const started = await startEngine()
      setIsRunning(started)
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
          <select className="rounded border bg-background px-2 py-1 text-sm">
            <option>ASIO: AsioBridge Virtual Driver</option>
          </select>
          <select
            className="rounded border bg-background px-2 py-1 text-sm"
            value={config?.sample_rate ?? 44100}
            onChange={(e) => handleSampleRateChange(Number(e.target.value))}
          >
            {SAMPLE_RATES.map((rate) => (
              <option key={rate} value={rate}>{rate} Hz</option>
            ))}
          </select>
          <select className="rounded border bg-background px-2 py-1 text-sm">
            {BIT_DEPTHS.map((bits) => (
              <option key={bits} value={bits}>{bits} bit</option>
            ))}
          </select>
        </div>

        <div className="mb-4 flex gap-4 overflow-x-auto pb-4">
          {racks.map((rack) => (
            <RackView key={rack.id} rack={rack} />
          ))}
        </div>

        <ConnectionMatrix connections={connections} />
      </main>

      <footer className="border-t bg-card px-4 py-2">
        <div className="flex items-center justify-between text-xs">
          <div className="flex items-center gap-4">
            <span className="text-muted-foreground">
              Sample Rate: {config?.sample_rate ?? '---'} Hz
            </span>
            <span className="text-muted-foreground">
              Bit Depth: {config?.bit_depth ?? '---'} bit
            </span>
            <span className="text-muted-foreground">
              Channels: {config?.channels ?? '---'}
            </span>
            <span className={isRunning ? 'text-green-500' : 'text-red-500'}>
              {isRunning ? '● Running' : '○ Stopped'}
            </span>
          </div>
          <Button size="sm" onClick={handleStartStop} variant={isRunning ? 'destructive' : 'default'}>
            {isRunning ? 'Stop' : 'Start'}
          </Button>
        </div>
      </footer>
    </div>
  )
}

export default App
