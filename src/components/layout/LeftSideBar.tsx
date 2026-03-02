import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Server,
  Bot,
  Terminal,
  Code,
  ChevronDown,
  Check,
  Cog,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { ActionDropdownMenuItem } from '@/components/ui/action-dropdown-menu-item'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { ChannelList, ChannelDialog } from '@/components/channels'
import { DroidFeatureList } from '@/components/droid'
import { OpenCodeFeatureList } from '@/components/opencode'
import { CodexFeatureList } from '@/components/codex'
import { OpenClawFeatureList } from '@/components/openclaw'
import { useUIStore } from '@/store/ui-store'
import { useChannelStore } from '@/store/channel-store'
import { useModelStore } from '@/store/model-store'
import type { Channel } from '@/lib/bindings'
import { saveChannelAuth } from '@/lib/channel-utils'

type NavigationView = 'droid' | 'channels' | 'opencode' | 'codex' | 'openclaw'

interface LeftSideBarProps {
  children?: React.ReactNode
  className?: string
}

export function LeftSideBar({ children, className }: LeftSideBarProps) {
  const { t } = useTranslation()
  const currentView = useUIStore(state => state.currentView)
  const setCurrentView = useUIStore(state => state.setCurrentView)
  const addChannel = useChannelStore(state => state.addChannel)
  const saveChannels = useChannelStore(state => state.saveChannels)
  const channels = useChannelStore(state => state.channels)
  const selectChannel = useChannelStore(state => state.selectChannel)
  const channelHasChanges = useChannelStore(state => state.hasChanges)
  const modelHasChanges = useModelStore(state => state.hasChanges)

  const [channelDialogOpen, setChannelDialogOpen] = useState(false)
  const [editingChannel, setEditingChannel] = useState<Channel | undefined>()
  const [pendingView, setPendingView] = useState<NavigationView | null>(null)
  const [dropdownOpen, setDropdownOpen] = useState(false)

  // Get lastToolView from store (automatically updated when switching to droid/opencode)
  const lastToolView = useUIStore(state => state.lastToolView)

  // Shared content for tool button
  const toolButtonContent = (
    <>
      {lastToolView === 'opencode' ? (
        <>
          <Terminal className="h-4 w-4 mr-2" />
          {t('sidebar.opencode')}
        </>
      ) : lastToolView === 'codex' ? (
        <>
          <Code className="h-4 w-4 mr-2" />
          {t('sidebar.codex')}
        </>
      ) : lastToolView === 'openclaw' ? (
        <>
          <Cog className="h-4 w-4 mr-2" />
          {t('sidebar.openclaw')}
        </>
      ) : (
        <>
          <Bot className="h-4 w-4 mr-2" />
          {t('sidebar.droid')}
        </>
      )}
      <ChevronDown className="h-3 w-3 ml-1" />
    </>
  )

  const handleViewChange = (view: NavigationView) => {
    if (view === currentView) return

    // Check if current view has unsaved changes
    const hasUnsavedChanges =
      (currentView === 'droid' && modelHasChanges) ||
      (currentView === 'channels' && channelHasChanges)

    if (hasUnsavedChanges) {
      setPendingView(view)
    } else {
      setCurrentView(view)
    }
  }

  const handleSaveAndSwitch = async () => {
    if (currentView === 'droid') {
      await useModelStore.getState().saveModels()
    } else if (currentView === 'channels') {
      await useChannelStore.getState().saveChannels()
    }
    if (pendingView) {
      setCurrentView(pendingView)
      setPendingView(null)
    }
  }

  const handleDiscardAndSwitch = () => {
    if (currentView === 'droid') {
      useModelStore.getState().resetChanges()
    } else if (currentView === 'channels') {
      useChannelStore.getState().resetChanges()
    }
    if (pendingView) {
      setCurrentView(pendingView)
      setPendingView(null)
    }
  }

  const handleAddChannel = () => {
    setEditingChannel(undefined)
    setChannelDialogOpen(true)
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
        <ActionButton
          variant={currentView === 'channels' ? 'secondary' : 'ghost'}
          size="sm"
          className="flex-1"
          onClick={() => handleViewChange('channels')}
        >
          <Server className="h-4 w-4 mr-2" />
          {t('sidebar.channels')}
        </ActionButton>

        {/* Droid/OpenCode Switcher - conditional rendering based on current view */}
        {currentView === 'channels' ? (
          // Simple button when in Channels - direct switch to lastToolView
          <ActionButton
            variant="ghost"
            size="sm"
            className="flex-1"
            onClick={() => handleViewChange(lastToolView)}
          >
            {toolButtonContent}
          </ActionButton>
        ) : (
          // Dropdown menu when in Droid/OpenCode - allow switching between tools
          <DropdownMenu open={dropdownOpen} onOpenChange={setDropdownOpen}>
            <DropdownMenuTrigger asChild>
              <ActionButton variant="secondary" size="sm" className="flex-1">
                {toolButtonContent}
              </ActionButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              <ActionDropdownMenuItem
                onClick={() => {
                  handleViewChange('droid')
                  setDropdownOpen(false)
                }}
              >
                <Bot className="h-4 w-4 mr-2" />
                {t('sidebar.droid')}
                {lastToolView === 'droid' && (
                  <Check className="h-4 w-4 ml-auto" />
                )}
              </ActionDropdownMenuItem>
              <ActionDropdownMenuItem
                onClick={() => {
                  handleViewChange('opencode')
                  setDropdownOpen(false)
                }}
              >
                <Terminal className="h-4 w-4 mr-2" />
                {t('sidebar.opencode')}
                {lastToolView === 'opencode' && (
                  <Check className="h-4 w-4 ml-auto" />
                )}
              </ActionDropdownMenuItem>
              <ActionDropdownMenuItem
                onClick={() => {
                  handleViewChange('codex')
                  setDropdownOpen(false)
                }}
              >
                <Code className="h-4 w-4 mr-2" />
                {t('sidebar.codex')}
                {lastToolView === 'codex' && (
                  <Check className="h-4 w-4 ml-auto" />
                )}
              </ActionDropdownMenuItem>
              <ActionDropdownMenuItem
                onClick={() => {
                  handleViewChange('openclaw')
                  setDropdownOpen(false)
                }}
              >
                <Cog className="h-4 w-4 mr-2" />
                {t('sidebar.openclaw')}
                {lastToolView === 'openclaw' && (
                  <Check className="h-4 w-4 ml-auto" />
                )}
              </ActionDropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>

      {/* Content based on view */}
      <div className="flex-1 overflow-hidden">
        {currentView === 'channels' ? (
          <ChannelList onAddChannel={handleAddChannel} />
        ) : currentView === 'droid' ? (
          <DroidFeatureList />
        ) : currentView === 'opencode' ? (
          <OpenCodeFeatureList />
        ) : currentView === 'openclaw' ? (
          <OpenClawFeatureList />
        ) : (
          <CodexFeatureList />
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

      {/* Unsaved Changes Confirmation Dialog */}
      <AlertDialog
        open={pendingView !== null}
        onOpenChange={open => !open && setPendingView(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('sidebar.unsavedChanges.title')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('sidebar.unsavedChanges.description')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <Button variant="destructive" onClick={handleDiscardAndSwitch}>
              {t('sidebar.unsavedChanges.discard')}
            </Button>
            <Button onClick={handleSaveAndSwitch}>
              {t('sidebar.unsavedChanges.save')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
