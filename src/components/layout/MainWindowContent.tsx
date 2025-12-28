import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import { ModelConfigPage } from '@/components/models'
import { DroidHelpersPage, SpecsPage } from '@/components/droid'
import { ChannelDetail, ChannelDialog } from '@/components/channels'
import { useUIStore } from '@/store/ui-store'
import { useChannelStore } from '@/store/channel-store'
import { commands, type Channel } from '@/lib/bindings'

interface MainWindowContentProps {
  children?: React.ReactNode
  className?: string
}

export function MainWindowContent({
  children,
  className,
}: MainWindowContentProps) {
  const { t } = useTranslation()
  const currentView = useUIStore(state => state.currentView)
  const droidSubView = useUIStore(state => state.droidSubView)
  const channels = useChannelStore(state => state.channels)
  const selectedChannelId = useChannelStore(state => state.selectedChannelId)
  const saveChannels = useChannelStore(state => state.saveChannels)

  const [editDialogOpen, setEditDialogOpen] = useState(false)

  const selectedChannel = channels.find(c => c.id === selectedChannelId)

  const handleEditChannel = () => {
    setEditDialogOpen(true)
  }

  const handleSaveChannel = async (
    channel: Channel,
    username: string,
    password: string
  ) => {
    await commands.saveChannelCredentials(channel.id, username, password)
    useChannelStore.getState().updateChannel(channel.id, channel)
    await saveChannels()
  }

  const renderContent = () => {
    if (children) return children

    if (currentView === 'droid') {
      if (droidSubView === 'models') {
        return <ModelConfigPage />
      }
      if (droidSubView === 'specs') {
        return <SpecsPage />
      }
      return <DroidHelpersPage />
    }

    // Channels view
    if (selectedChannel) {
      return (
        <>
          <ChannelDetail channel={selectedChannel} onEdit={handleEditChannel} />
          <ChannelDialog
            open={editDialogOpen}
            onOpenChange={setEditDialogOpen}
            channel={selectedChannel}
            onSave={handleSaveChannel}
          />
        </>
      )
    }

    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <p>{t('channels.selectChannelHint')}</p>
      </div>
    )
  }

  return (
    <div className={cn('flex h-full flex-col bg-background', className)}>
      {renderContent()}
    </div>
  )
}
