import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Server, Download, Upload } from 'lucide-react'
import { save, open } from '@tauri-apps/plugin-dialog'
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs'
import { toast } from 'sonner'
import { ActionButton } from '@/components/ui/action-button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
import { useChannelStore } from '@/store/channel-store'
import { commands, type Channel, type ChannelType } from '@/lib/bindings'
import { ChannelExportDialog } from './ChannelExportDialog'
import {
  ChannelImportDialog,
  type ChannelExportEntry,
  type ChannelMergeStrategy,
} from './ChannelImportDialog'

interface ChannelExportData {
  version: number
  exportedAt: string
  channels: ChannelExportEntry[]
}

const validChannelTypes: ChannelType[] = [
  'new-api',
  'sub-2-api',
  'cli-proxy-api',
  'ollama',
  'general',
]

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
  const { t } = useTranslation()
  return (
    <ActionButton
      variant="ghost"
      onClick={onClick}
      className={cn(
        'w-full flex items-center gap-2 px-3 py-2 text-sm rounded-md transition-colors text-start justify-start h-auto',
        isSelected
          ? 'bg-accent text-accent-foreground'
          : 'hover:bg-accent/50 text-muted-foreground hover:text-foreground'
      )}
    >
      <Server className="h-4 w-4 shrink-0" />
      <span className="truncate">{channel.name}</span>
      {!channel.enabled && (
        <span className="ml-auto text-xs text-muted-foreground">
          ({t('common.disabled')})
        </span>
      )}
    </ActionButton>
  )
}

export function ChannelList({ onAddChannel }: ChannelListProps) {
  const { t } = useTranslation()
  const channels = useChannelStore(state => state.channels)
  const selectedChannelId = useChannelStore(state => state.selectedChannelId)
  const loadChannels = useChannelStore(state => state.loadChannels)
  const addChannel = useChannelStore(state => state.addChannel)
  const updateChannel = useChannelStore(state => state.updateChannel)
  const saveChannels = useChannelStore(state => state.saveChannels)
  const selectChannel = useChannelStore(state => state.selectChannel)
  const setError = useChannelStore(state => state.setError)

  const [exportDialogOpen, setExportDialogOpen] = useState(false)
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [importChannels, setImportChannels] = useState<ChannelExportEntry[]>([])

  useEffect(() => {
    loadChannels()
  }, [loadChannels])

  const handleExport = async (includeCredentials: boolean) => {
    if (channels.length === 0) return

    try {
      const filePath = await save({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        defaultPath: 'channels-export.json',
      })
      if (!filePath) return

      const entries: ChannelExportEntry[] = []
      for (const ch of channels) {
        const entry: ChannelExportEntry = {
          id: ch.id,
          name: ch.name,
          type: ch.type,
          baseUrl: ch.baseUrl,
          enabled: ch.enabled,
          createdAt: ch.createdAt,
        }

        if (includeCredentials) {
          const isApiKeyAuth =
            ch.type === 'cli-proxy-api' ||
            ch.type === 'ollama' ||
            ch.type === 'general'

          if (isApiKeyAuth) {
            const result = await commands.getChannelApiKey(ch.id)
            if (result.status === 'ok' && result.data) {
              entry.apiKey = result.data
            }
          } else {
            const result = await commands.getChannelCredentials(ch.id)
            if (result.status === 'ok' && result.data) {
              entry.credentials = {
                username: result.data[0],
                password: result.data[1],
              }
            }
          }
        }

        entries.push(entry)
      }

      const exportData: ChannelExportData = {
        version: 1,
        exportedAt: new Date().toISOString(),
        channels: entries,
      }

      await writeTextFile(filePath, JSON.stringify(exportData, null, 2))
      toast.success(t('channels.export.success'))
    } catch (e) {
      setError(String(e))
    }
  }

  const handleImport = async () => {
    try {
      const filePath = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      })
      if (!filePath) return

      const content = await readTextFile(filePath)
      const data = JSON.parse(content) as ChannelExportData

      if (!data.channels || !Array.isArray(data.channels)) {
        setError(t('channels.import.invalidFormat'))
        return
      }

      const validEntries = data.channels.filter(
        ch =>
          typeof ch.name === 'string' &&
          typeof ch.baseUrl === 'string' &&
          typeof ch.type === 'string' &&
          validChannelTypes.includes(ch.type as ChannelType)
      )

      if (validEntries.length === 0) {
        setError(t('channels.import.noValidChannels'))
        return
      }

      setImportChannels(validEntries)
      setImportDialogOpen(true)
    } catch (e) {
      setError(String(e))
    }
  }

  const handleImportConfirm = async (
    selectedEntries: ChannelExportEntry[],
    strategy: ChannelMergeStrategy
  ) => {
    const findDuplicateId = (entry: ChannelExportEntry) =>
      channels.find(
        existing =>
          existing.name === entry.name &&
          existing.baseUrl === entry.baseUrl &&
          existing.type === entry.type
      )?.id

    for (const entry of selectedEntries) {
      const dupId = findDuplicateId(entry)

      if (!dupId) {
        const newChannel: Channel = {
          id: crypto.randomUUID(),
          name: entry.name,
          type: entry.type as ChannelType,
          baseUrl: entry.baseUrl,
          enabled: entry.enabled,
          createdAt: entry.createdAt ?? Date.now(),
        }
        addChannel(newChannel)
        await saveCredentialsForEntry(newChannel.id, entry)
      } else {
        switch (strategy) {
          case 'skip':
            break
          case 'replace':
            updateChannel(dupId, {
              name: entry.name,
              type: entry.type as ChannelType,
              baseUrl: entry.baseUrl,
              enabled: entry.enabled,
            })
            await saveCredentialsForEntry(dupId, entry)
            break
          case 'keep-both': {
            const newChannel: Channel = {
              id: crypto.randomUUID(),
              name: entry.name,
              type: entry.type as ChannelType,
              baseUrl: entry.baseUrl,
              enabled: entry.enabled,
              createdAt: entry.createdAt ?? Date.now(),
            }
            addChannel(newChannel)
            await saveCredentialsForEntry(newChannel.id, entry)
            break
          }
        }
      }
    }

    await saveChannels()
    setImportDialogOpen(false)
    setImportChannels([])
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-3 py-2 border-b">
        <span className="text-sm font-medium">{t('channels.title')}</span>
        <div className="flex items-center gap-1">
          <ActionButton
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={handleImport}
            title={t('channels.import.button')}
          >
            <Upload className="h-4 w-4" />
          </ActionButton>
          <ActionButton
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => setExportDialogOpen(true)}
            disabled={channels.length === 0}
            title={t('channels.export.button')}
          >
            <Download className="h-4 w-4" />
          </ActionButton>
          <ActionButton
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={onAddChannel}
          >
            <Plus className="h-4 w-4" />
          </ActionButton>
        </div>
      </div>
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {channels.length === 0 ? (
            <p className="text-sm text-muted-foreground px-3 py-2">
              {t('channels.noChannels')}
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

      <ChannelExportDialog
        open={exportDialogOpen}
        onOpenChange={setExportDialogOpen}
        onExport={handleExport}
      />

      <ChannelImportDialog
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        importChannels={importChannels}
        existingChannels={channels}
        onImport={handleImportConfirm}
      />
    </div>
  )
}

async function saveCredentialsForEntry(
  channelId: string,
  entry: ChannelExportEntry
) {
  const isApiKeyAuth =
    entry.type === 'cli-proxy-api' ||
    entry.type === 'ollama' ||
    entry.type === 'general'

  if (isApiKeyAuth && entry.apiKey) {
    await commands.saveChannelApiKey(channelId, entry.apiKey)
  } else if (!isApiKeyAuth && entry.credentials) {
    await commands.saveChannelCredentials(
      channelId,
      entry.credentials.username,
      entry.credentials.password
    )
  }
}
