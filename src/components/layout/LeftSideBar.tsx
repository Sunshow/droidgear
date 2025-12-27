import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Server, Bot } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { ChannelList, ChannelDialog } from '@/components/channels'
import { DroidFeatureList } from '@/components/droid'
import { useUIStore } from '@/store/ui-store'
import { useChannelStore } from '@/store/channel-store'
import { useModelStore } from '@/store/model-store'
import { commands, type Channel } from '@/lib/bindings'

type NavigationView = 'droid' | 'channels'

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
    } else {
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
    } else {
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

    // Auto refresh tokens after saving channel
    useChannelStore
      .getState()
      .fetchKeys(channel.id, channel.type, channel.baseUrl)
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
          onClick={() => handleViewChange('channels')}
        >
          <Server className="h-4 w-4 mr-2" />
          {t('sidebar.channels')}
        </Button>
        <Button
          variant={currentView === 'droid' ? 'secondary' : 'ghost'}
          size="sm"
          className="flex-1"
          onClick={() => handleViewChange('droid')}
        >
          <Bot className="h-4 w-4 mr-2" />
          {t('sidebar.droid')}
        </Button>
      </div>

      {/* Content based on view */}
      <div className="flex-1 overflow-hidden">
        {currentView === 'channels' ? (
          <ChannelList onAddChannel={handleAddChannel} />
        ) : (
          <DroidFeatureList />
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
