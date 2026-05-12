import React from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'

type RackType = 'asio-driver-in' | 'asio-driver-out' | 'asio-host-in' | 'network-in' | 'network-out' | 'looper-in' | 'looper-out' | 'wdm-in' | 'mix-out'

interface Channel {
  id: number
  name: string
  active: boolean
  level: number
}

interface Rack {
  id: RackType
  name: string
  channels: Channel[]
}

const RACKS: Rack[] = [
  {
    id: 'asio-driver-in',
    name: 'ASIO Driver IN',
    channels: Array.from({ length: 8 }, (_, i) => ({
      id: i,
      name: `IN ${i + 1}`,
      active: true,
      level: 0.8,
    })),
  },
  {
    id: 'asio-driver-out',
    name: 'ASIO Driver OUT/MIX',
    channels: Array.from({ length: 8 }, (_, i) => ({
      id: i,
      name: `OUT ${i + 1}`,
      active: true,
      level: 0.8,
    })),
  },
  {
    id: 'asio-host-in',
    name: 'ASIO Host IN/MIX',
    channels: Array.from({ length: 8 }, (_, i) => ({
      id: i,
      name: `HOST ${i + 1}`,
      active: true,
      level: 0.8,
    })),
  },
  {
    id: 'network-in',
    name: 'Network IN',
    channels: Array.from({ length: 4 }, (_, i) => ({
      id: i,
      name: `NET-IN ${i + 1}`,
      active: false,
      level: 0.0,
    })),
  },
  {
    id: 'network-out',
    name: 'Network OUT',
    channels: Array.from({ length: 4 }, (_, i) => ({
      id: i,
      name: `NET-OUT ${i + 1}`,
      active: false,
      level: 0.0,
    })),
  },
  {
    id: 'wdm-in',
    name: 'WDM IN',
    channels: Array.from({ length: 8 }, (_, i) => ({
      id: i,
      name: `WDM ${i + 1}`,
      active: false,
      level: 0.0,
    })),
  },
  {
    id: 'mix-out',
    name: 'Mix OUT',
    channels: Array.from({ length: 8 }, (_, i) => ({
      id: i,
      name: `MIX ${i + 1}`,
      active: true,
      level: 0.8,
    })),
  },
]

const RACK_COLORS: Record<RackType, string> = {
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

function ChannelStrip({ channel, rackId }: { channel: Channel; rackId: RackType }) {
  return (
    <div className="flex flex-col items-center gap-2">
      <span className="text-xs text-muted-foreground">{channel.name}</span>
      <div className="flex h-48 flex-col items-center gap-1">
        <Slider
          value={[channel.level]}
          max={1}
          step={0.01}
          className="h-48 rotate-180"
          onValueChange={([v]) => {
            console.log(`[${rackId}] Ch${channel.id} level: ${v}`)
          }}
        />
        <span className="text-xs text-muted-foreground">{Math.round(channel.level * 100)}%</span>
      </div>
      <Switch
        checked={channel.active}
        onCheckedChange={(checked) => {
          console.log(`[${rackId}] Ch${channel.id} ${checked ? 'ON' : 'OFF'}`)
        }}
      />
    </div>
  )
}

function RackView({ rack }: { rack: Rack }) {
  const colorClass = RACK_COLORS[rack.id] || 'border-gray-500'

  return (
    <Card className={cn('w-48', colorClass)}>
      <CardHeader className="p-3">
        <CardTitle className="text-xs">{rack.name}</CardTitle>
      </CardHeader>
      <CardContent className="flex justify-between p-3">
        {rack.channels.map((ch) => (
          <ChannelStrip key={ch.id} channel={ch} rackId={rack.id} />
        ))}
      </CardContent>
    </Card>
  )
}

function ConnectionMatrix() {
  return (
    <Card>
      <CardHeader className="p-3">
        <CardTitle className="text-sm">Connection Matrix</CardTitle>
      </CardHeader>
      <CardContent className="p-3">
        <div className="grid grid-cols-7 gap-1 text-center text-xs">
          <div></div>
          {RACKS.slice(0, 6).map((r) => (
            <div key={r.id} className="text-muted-foreground">{r.name.split(' ')[0]}</div>
          ))}
          {RACKS.slice(0, 6).map((src) => (
            <React.Fragment key={src.id}>
              <div className="text-muted-foreground">{src.name.split(' ')[0]}</div>
              {RACKS.slice(0, 6).map((dst) => (
                <div key={dst.id} className="h-8 w-8 rounded bg-secondary">
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-full w-full p-0 text-[8px]"
                  >
                    {src.id === dst.id ? 'X' : '→'}
                  </Button>
                </div>
              ))}
            </React.Fragment>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}

function StatusBar() {
  return (
    <div className="flex items-center justify-between border-t bg-card px-4 py-2 text-xs">
      <div className="flex items-center gap-4">
        <span className="text-muted-foreground">Sample Rate: 44100 Hz</span>
        <span className="text-muted-foreground">Bit Depth: 24 bit</span>
        <span className="text-muted-foreground">Channels: 2</span>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="outline" size="sm">Save Profile</Button>
        <Button variant="outline" size="sm">Load Profile</Button>
        <Button size="sm">Start</Button>
      </div>
    </div>
  )
}

function App() {
  return (
    <div className="min-h-screen bg-background">
      <header className="border-b bg-card px-4 py-3">
        <div className="flex items-center justify-between">
          <h1 className="text-lg font-bold">AsioBridge</h1>
          <div className="flex items-center gap-2">
            <select className="rounded border bg-background px-2 py-1 text-sm">
              <option>Profile 1</option>
              <option>Profile 2</option>
              <option>Profile 3</option>
              <option>Profile 4</option>
              <option>Profile 5</option>
              <option>Profile 6</option>
              <option>Profile 7</option>
              <option>Profile 8</option>
            </select>
          </div>
        </div>
      </header>

      <main className="p-4">
        <div className="mb-4 flex gap-2">
          <select className="rounded border bg-background px-2 py-1 text-sm">
            <option>ASIO: AsioBridge Virtual Driver</option>
            <option>ASIO: Steinberg ASIO</option>
          </select>
          <select className="rounded border bg-background px-2 py-1 text-sm">
            <option>44100 Hz</option>
            <option>48000 Hz</option>
            <option>88200 Hz</option>
            <option>96000 Hz</option>
          </select>
        </div>

        <div className="mb-4 flex gap-4 overflow-x-auto">
          {RACKS.map((rack) => (
            <RackView key={rack.id} rack={rack} />
          ))}
        </div>

        <ConnectionMatrix />
      </main>

      <StatusBar />
    </div>
  )
}

export default App
