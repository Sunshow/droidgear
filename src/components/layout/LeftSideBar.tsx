import { useState } from 'react'
import { Server, Cpu } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { ChannelList, ChannelDialog } from '@/components/channels'
import { useUIStore } from '@/store/ui-store'
import { useChannelStore } from '@/store/channel-store'
import { commands, type Channel } from '@/lib/bindings'

interface LeftSideBarProps {
  children?: React.ReactNode
  className?: string
}

export function LeftSideBar({ children, className }: LeftSideBarProps) {
  const currentView = useUIStore(state => state.currentView)
  const setCurrentView = useUIStore(state => state.setCurrentView)
  const addChannel = useChannelStore(state => state.addChannel)
  const saveChannels = useChannelStore(state => state.saveChannels)
  const channels = useChannelStore(state => state.channels)
  const selectChannel = useChannelStore(state => state.selectChannel)

  const [channelDialogOpen, setChannelDialogOpen] = useState(false)
  const [editingChannel, setEditingChannel] = useState<Channel | undefined>()

  const handleAddChannel = () => {
    setEditingChannel(undefined)
    setChannelDialogOpen(true)
  }

  const handleSaveChannel = async (
    channel: Channel,
    username: string,
    password: string
  ) => {
    // Save credentials to storage
    console.log(
      'Saving credentials for channel:',
      channel.id,
      'username:',
      username
    )
    const credResult = await commands.saveChannelCredentials(
      channel.id,
      username,
      password
    )
    console.log('Save credentials result:', credResult)

    if (credResult.status !== 'ok') {
      console.error('Failed to save credentials:', credResult.error)
      return
    }

    // Check if this is an edit or new channel
    const existingIndex = channels.findIndex(c => c.id === channel.id)
    if (existingIndex >= 0) {
      useChannelStore.getState().updateChannel(channel.id, channel)
    } else {
      addChannel(channel)
    }

    await saveChannels()
    selectChannel(channel.id)
    setCurrentView('channels')
  }

  return (
    <div
      className={cn('flex h-full flex-col border-r bg-background', className)}
    >
      {/* Navigation Tabs */}
      <div className="flex items-center gap-1 p-2 border-b">
        <Button
          variant={currentView === 'channels' ? 'secondary' : 'ghost'}
          size="sm"
          className="flex-1"
          onClick={() => setCurrentView('channels')}
        >
          <Server className="h-4 w-4 mr-2" />
          Channels
        </Button>
        <Button
          variant={currentView === 'models' ? 'secondary' : 'ghost'}
          size="sm"
          className="flex-1"
          onClick={() => setCurrentView('models')}
        >
          <Cpu className="h-4 w-4 mr-2" />
          Models
        </Button>
      </div>

      {/* Content based on view */}
      <div className="flex-1 overflow-hidden">
        {currentView === 'channels' ? (
          <ChannelList onAddChannel={handleAddChannel} />
        ) : (
          <div className="p-3 text-sm text-muted-foreground">
            Select a model from the main panel to edit
          </div>
        )}
      </div>

      {children}

      {/* Channel Dialog */}
      <ChannelDialog
        open={channelDialogOpen}
        onOpenChange={setChannelDialogOpen}
        channel={editingChannel}
        onSave={handleSaveChannel}
      />
    </div>
  )
}
