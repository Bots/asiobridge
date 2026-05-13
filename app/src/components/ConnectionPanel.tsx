import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Switch } from '@/components/ui/switch'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { useEngine } from '@/hooks/useEngine'
import type { ConnectionState } from '@/types/engine'

const CONNECTION_TYPE_COLORS: Record<string, string> = {
  direct: 'bg-blue-500',
  network: 'bg-green-500',
  wdm: 'bg-purple-500',
  null: 'bg-gray-500',
  multi_client: 'bg-yellow-500',
  vst: 'bg-pink-500',
  midi: 'bg-orange-500',
}

function ConnectionRow({ connection, onToggle }: { connection: ConnectionState; onToggle: () => void }) {
  const typeColor = CONNECTION_TYPE_COLORS[connection.connection_type] || 'bg-gray-500'

  return (
    <div className={cn(
      'flex items-center justify-between p-3 rounded-lg border transition-colors',
      connection.is_active ? 'border-primary/50 bg-primary/5' : 'border-border'
    )}>
      <div className="flex items-center gap-3">
        <div className={cn('w-3 h-3 rounded-full shrink-0', typeColor)} />
        <div className="min-w-0">
          <div className="text-sm font-medium truncate">{connection.source_rack}</div>
          <div className="text-xs text-muted-foreground truncate">
            → {connection.dest_rack}
          </div>
        </div>
      </div>
      <div className="flex items-center gap-2 shrink-0">
        <Badge variant="outline" className="text-xs">{connection.connection_type}</Badge>
        <div className="text-[10px] text-muted-foreground">
          {connection.source_channel}→{connection.dest_channel}
        </div>
        <Switch checked={connection.is_active} onCheckedChange={onToggle} />
      </div>
    </div>
  )
}

export function ConnectionPanel({ connections }: { connections: ConnectionState[] }) {
  const [filter, setFilter] = useState<string>('all')
  const [search, setSearch] = useState('')
  const { toggleConnection } = useEngine()

  const handleToggle = async (connection: ConnectionState) => {
    try {
      await toggleConnection(
        connection.source_rack,
        connection.source_channel,
        connection.dest_rack,
        connection.dest_channel
      )
    } catch (e) {
      console.error('Failed to toggle connection:', e)
    }
  }

  const filtered = connections.filter((c) => {
    if (filter !== 'all' && c.connection_type !== filter) return false
    if (search && !c.source_rack.toLowerCase().includes(search.toLowerCase()) &&
        !c.dest_rack.toLowerCase().includes(search.toLowerCase())) return false
    return true
  })

  const types = [...new Set(connections.map((c) => c.connection_type))]

  return (
    <Card>
      <CardHeader className="p-3">
        <CardTitle className="text-sm">Connections ({connections.length})</CardTitle>
      </CardHeader>
      <CardContent className="p-3">
        <div className="flex gap-2 mb-3">
          <Input
            placeholder="Search connections..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="h-8"
          />
          <Select value={filter} onValueChange={setFilter}>
            <SelectTrigger className="w-32 h-8">
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All</SelectItem>
              {types.map((type) => (
                <SelectItem key={type} value={type}>{type}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col gap-2">
          {filtered.map((connection) => (
            <ConnectionRow
              key={`${connection.source_rack}-${connection.source_channel}-${connection.dest_rack}-${connection.dest_channel}`}
              connection={connection}
              onToggle={() => handleToggle(connection)}
            />
          ))}
          {filtered.length === 0 && (
            <div className="text-center py-8 text-sm text-muted-foreground">
              No connections found
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
