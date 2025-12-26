import { useEffect } from 'react'
import { Plus, Server } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
import { useChannelStore } from '@/store/channel-store'
import type { Channel } from '@/lib/bindings'

interface ChannelListProps {
  onAddChannel: () => void
}

function ChannelItem({
  channel,
  isSelected,
  onClick,
}: {
  channel: Channel
  isSelected: boolean
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'w-full flex items-center gap-2 px-3 py-2 text-sm rounded-md transition-colors text-start',
        isSelected
          ? 'bg-accent text-accent-foreground'
          : 'hover:bg-accent/50 text-muted-foreground hover:text-foreground'
      )}
    >
      <Server className="h-4 w-4 shrink-0" />
      <span className="truncate">{channel.name}</span>
      {!channel.enabled && (
        <span className="ml-auto text-xs text-muted-foreground">
          (disabled)
        </span>
      )}
    </button>
  )
}

export function ChannelList({ onAddChannel }: ChannelListProps) {
  const channels = useChannelStore(state => state.channels)
  const selectedChannelId = useChannelStore(state => state.selectedChannelId)
  const loadChannels = useChannelStore(state => state.loadChannels)
  const selectChannel = useChannelStore(state => state.selectChannel)

  useEffect(() => {
    loadChannels()
  }, [loadChannels])

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-3 py-2 border-b">
        <span className="text-sm font-medium">Channels</span>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={onAddChannel}
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {channels.length === 0 ? (
            <p className="text-sm text-muted-foreground px-3 py-2">
              No channels configured
            </p>
          ) : (
            channels.map(channel => (
              <ChannelItem
                key={channel.id}
                channel={channel}
                isSelected={selectedChannelId === channel.id}
                onClick={() => selectChannel(channel.id)}
              />
            ))
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
