import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import { ModelConfigPage } from '@/components/models'
import {
  DroidHelpersPage,
  LegacyVersionsPage,
  SpecsPage,
  McpPage,
  SessionsPage,
  TerminalPage,
} from '@/components/droid'
import { OpenCodeConfigPage } from '@/components/opencode'
import { CodexConfigPage } from '@/components/codex'
import { OpenClawConfigPage, OpenClawHelpersPage } from '@/components/openclaw'
import { ChannelDetail, ChannelDialog } from '@/components/channels'
import { useUIStore } from '@/store/ui-store'
import { useChannelStore } from '@/store/channel-store'
import type { Channel } from '@/lib/bindings'
import { saveChannelAuth } from '@/lib/channel-utils'

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
  const openclawSubView = useUIStore(state => state.openclawSubView)
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
    const authResult = await saveChannelAuth(
      channel.id,
      channel.type,
      username,
      password
    )
    if (!authResult.ok) {
      console.error('Failed to save channel auth:', authResult.error)
      return
    }
    useChannelStore.getState().updateChannel(channel.id, channel)
    await saveChannels()
  }

  const renderContent = () => {
    if (children) return children

    if (currentView === 'droid') {
      return (
        <>
          {droidSubView === 'models' && <ModelConfigPage />}
          {droidSubView === 'specs' && <SpecsPage />}
          {droidSubView === 'mcp' && <McpPage />}
          {droidSubView === 'sessions' && <SessionsPage />}
          {droidSubView === 'helpers' && <DroidHelpersPage />}
          {droidSubView === 'legacy-versions' && <LegacyVersionsPage />}
        </>
      )
    }

    if (currentView === 'opencode') {
      return <OpenCodeConfigPage />
    }

    if (currentView === 'codex') {
      return <CodexConfigPage />
    }

    if (currentView === 'openclaw') {
      return (
        <>
          {openclawSubView === 'providers' && <OpenClawConfigPage />}
          {openclawSubView === 'helpers' && <OpenClawHelpersPage />}
        </>
      )
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
    <div
      className={cn('flex h-full flex-col bg-background relative', className)}
    >
      {renderContent()}
      {/* Terminal is always mounted across all views, hidden when not active */}
      <div
        className={cn(
          !(currentView === 'droid' && droidSubView === 'terminal') && 'hidden',
          'absolute inset-0'
        )}
      >
        <TerminalPage />
      </div>
    </div>
  )
}
