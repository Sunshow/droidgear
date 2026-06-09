import { Fragment, Suspense, lazy, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'
import { Loader2 } from 'lucide-react'

// Code-split all config pages — only loaded when the user navigates to that view
// Note: Import from individual files (not barrel indexes) to avoid conflicts with
// static barrel imports in LeftSideBar, which would prevent effective code-splitting.
const ModelConfigPage = lazy(() =>
  import('@/components/models/ModelConfigPage').then(m => ({
    default: m.ModelConfigPage,
  }))
)
const DroidSettingsPage = lazy(() =>
  import('@/components/droid/DroidSettingsPage').then(m => ({
    default: m.DroidSettingsPage,
  }))
)
const LegacyVersionsPage = lazy(() =>
  import('@/components/droid/LegacyVersionsPage').then(m => ({
    default: m.LegacyVersionsPage,
  }))
)
const SpecsPage = lazy(() =>
  import('@/components/droid/SpecsPage').then(m => ({ default: m.SpecsPage }))
)
const McpPage = lazy(() =>
  import('@/components/droid/McpPage').then(m => ({ default: m.McpPage }))
)
const SessionsPage = lazy(() =>
  import('@/components/droid/SessionsPage').then(m => ({
    default: m.SessionsPage,
  }))
)
const TerminalPage = lazy(() =>
  import('@/components/droid/TerminalPage').then(m => ({
    default: m.TerminalPage,
  }))
)
const MissionsPage = lazy(() =>
  import('@/components/droid/MissionsPage').then(m => ({
    default: m.MissionsPage,
  }))
)
const FactoryAuthPage = lazy(() =>
  import('@/components/factory-auth/FactoryAuthPage').then(m => ({
    default: m.FactoryAuthPage,
  }))
)
const OpenCodeConfigPage = lazy(() =>
  import('@/components/opencode/OpenCodeConfigPage').then(m => ({
    default: m.OpenCodeConfigPage,
  }))
)
const CodexConfigPage = lazy(() =>
  import('@/components/codex/CodexConfigPage').then(m => ({
    default: m.CodexConfigPage,
  }))
)
const ClaudeSettingsPage = lazy(() =>
  import('@/components/claude/ClaudeSettingsPage').then(m => ({
    default: m.ClaudeSettingsPage,
  }))
)
const OpenClawConfigPage = lazy(() =>
  import('@/components/openclaw/OpenClawConfigPage').then(m => ({
    default: m.OpenClawConfigPage,
  }))
)
const OpenClawHelpersPage = lazy(() =>
  import('@/components/openclaw/OpenClawHelpersPage').then(m => ({
    default: m.OpenClawHelpersPage,
  }))
)
const SubagentsPage = lazy(() =>
  import('@/components/openclaw/SubagentsPage').then(m => ({
    default: m.SubagentsPage,
  }))
)
const HermesConfigPage = lazy(() =>
  import('@/components/hermes/HermesConfigPage').then(m => ({
    default: m.HermesConfigPage,
  }))
)
const PiConfigPage = lazy(() =>
  import('@/components/pi/PiConfigPage').then(m => ({
    default: m.PiConfigPage,
  }))
)
const ChannelDetail = lazy(() =>
  import('@/components/channels/ChannelDetail').then(m => ({
    default: m.ChannelDetail,
  }))
)
const ExportTemplatesPage = lazy(() =>
  import('@/components/export/ExportTemplatesPage').then(m => ({
    default: m.ExportTemplatesPage,
  }))
)
// ChannelDialog is also statically imported by LeftSideBar, so lazy loading
// here won't create a separate chunk. Keep it as a regular static import.
import { ChannelDialog } from '@/components/channels/ChannelDialog'

function PageLoadingFallback() {
  return (
    <div className="flex h-full items-center justify-center">
      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
    </div>
  )
}
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
  const droidRefreshKey = useUIStore(state => state.droidRefreshKey)
  const openclawSubView = useUIStore(state => state.openclawSubView)
  const channelsSubView = useUIStore(state => state.channelsSubView)
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
        <Fragment key={droidRefreshKey}>
          {droidSubView === 'models' && <ModelConfigPage />}
          {droidSubView === 'specs' && <SpecsPage />}
          {droidSubView === 'mcp' && <McpPage />}
          {droidSubView === 'sessions' && <SessionsPage />}
          {droidSubView === 'settings' && <DroidSettingsPage />}
          {droidSubView === 'auth-profiles' && <FactoryAuthPage />}
          {droidSubView === 'missions' && <MissionsPage />}
          {droidSubView === 'legacy-versions' && <LegacyVersionsPage />}
        </Fragment>
      )
    }

    if (currentView === 'opencode') {
      return <OpenCodeConfigPage />
    }

    if (currentView === 'codex') {
      return <CodexConfigPage />
    }

    if (currentView === 'claude') {
      return <ClaudeSettingsPage />
    }

    if (currentView === 'hermes') {
      return <HermesConfigPage />
    }

    if (currentView === 'pi') {
      return <PiConfigPage />
    }

    if (currentView === 'openclaw') {
      return (
        <>
          {openclawSubView === 'providers' && <OpenClawConfigPage />}
          {openclawSubView === 'subagents' && <SubagentsPage />}
          {openclawSubView === 'helpers' && <OpenClawHelpersPage />}
        </>
      )
    }

    // Channels view
    if (channelsSubView === 'export-templates') {
      return <ExportTemplatesPage />
    }

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
      <Suspense fallback={<PageLoadingFallback />}>{renderContent()}</Suspense>
      {/* Terminal is always mounted across all views, hidden when not active */}
      <div
        className={cn(
          !(currentView === 'droid' && droidSubView === 'terminal') && 'hidden',
          'absolute inset-0'
        )}
      >
        <Suspense fallback={<PageLoadingFallback />}>
          <TerminalPage />
        </Suspense>
      </div>
    </div>
  )
}
