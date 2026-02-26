import { useState, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertCircle } from 'lucide-react'
import {
  ResizableDialog,
  ResizableDialogContent,
  ResizableDialogDescription,
  ResizableDialogHeader,
  ResizableDialogBody,
  ResizableDialogTitle,
  ResizableDialogFooter,
} from '@/components/ui/resizable-dialog'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { Channel } from '@/lib/bindings'

export type ChannelMergeStrategy = 'skip' | 'replace' | 'keep-both'

export interface ChannelExportEntry {
  id: string
  name: string
  type: string
  baseUrl: string
  enabled: boolean
  createdAt: number
  credentials?: { username: string; password: string }
  apiKey?: string
}

interface ChannelImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  importChannels: ChannelExportEntry[]
  existingChannels: Channel[]
  onImport: (
    channels: ChannelExportEntry[],
    strategy: ChannelMergeStrategy
  ) => void
}

function isDuplicate(
  channel: ChannelExportEntry,
  existingChannels: Channel[]
): boolean {
  return existingChannels.some(
    existing =>
      existing.name === channel.name &&
      existing.baseUrl === channel.baseUrl &&
      existing.type === channel.type
  )
}

export function ChannelImportDialog({
  open,
  onOpenChange,
  importChannels,
  existingChannels,
  onImport,
}: ChannelImportDialogProps) {
  const { t } = useTranslation()
  const [selectedIndices, setSelectedIndices] = useState<Set<number>>(
    () => new Set(importChannels.map((_, i) => i))
  )
  const [mergeStrategy, setMergeStrategy] =
    useState<ChannelMergeStrategy>('skip')

  const duplicateIndices = useMemo(() => {
    const indices = new Set<number>()
    importChannels.forEach((channel, index) => {
      if (isDuplicate(channel, existingChannels)) {
        indices.add(index)
      }
    })
    return indices
  }, [importChannels, existingChannels])

  const handleSelectAll = () => {
    setSelectedIndices(new Set(importChannels.map((_, i) => i)))
  }

  const handleDeselectAll = () => {
    setSelectedIndices(new Set())
  }

  const handleToggle = (index: number) => {
    setSelectedIndices(prev => {
      const next = new Set(prev)
      if (next.has(index)) {
        next.delete(index)
      } else {
        next.add(index)
      }
      return next
    })
  }

  const handleImport = () => {
    const selected = importChannels.filter((_, i) => selectedIndices.has(i))
    onImport(selected, mergeStrategy)
    onOpenChange(false)
  }

  const selectedCount = selectedIndices.size
  const duplicateSelectedCount = Array.from(selectedIndices).filter(i =>
    duplicateIndices.has(i)
  ).length
  const hasCredentials = importChannels.some(ch => ch.credentials || ch.apiKey)

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={700}
        defaultHeight={500}
        minWidth={500}
        minHeight={350}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>
            {t('channels.import.title')}
          </ResizableDialogTitle>
          <ResizableDialogDescription>
            {t('channels.import.description')}
            {hasCredentials && (
              <span className="block mt-1 text-green-600 dark:text-green-400">
                {t('channels.import.hasCredentials')}
              </span>
            )}
          </ResizableDialogDescription>
        </ResizableDialogHeader>

        <ResizableDialogBody>
          <div className="flex flex-wrap items-center gap-2 pb-2">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">
                {t('channels.import.mergeStrategy')}:
              </span>
              <Select
                value={mergeStrategy}
                onValueChange={v => setMergeStrategy(v as ChannelMergeStrategy)}
              >
                <SelectTrigger className="w-[180px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="skip">
                    {t('channels.import.strategySkip')}
                  </SelectItem>
                  <SelectItem value="replace">
                    {t('channels.import.strategyReplace')}
                  </SelectItem>
                  <SelectItem value="keep-both">
                    {t('channels.import.strategyKeepBoth')}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="flex-1" />
            <div className="flex items-center gap-2">
              <Button variant="outline" size="sm" onClick={handleSelectAll}>
                {t('common.selectAll')}
              </Button>
              <Button variant="outline" size="sm" onClick={handleDeselectAll}>
                {t('common.deselectAll')}
              </Button>
            </div>
          </div>

          {duplicateSelectedCount > 0 && (
            <div className="flex items-center gap-2 p-2 mb-2 bg-yellow-50 dark:bg-yellow-900/20 rounded-md text-sm">
              <AlertCircle className="h-4 w-4 text-yellow-600 dark:text-yellow-400" />
              <span className="text-yellow-700 dark:text-yellow-300">
                {t('channels.import.duplicateWarning', {
                  count: duplicateSelectedCount,
                })}
              </span>
            </div>
          )}

          <div className="flex-1 overflow-y-auto border rounded-md">
            <div className="p-2 space-y-1">
              {importChannels.map((channel, index) => {
                const isDup = duplicateIndices.has(index)
                return (
                  <div
                    key={index}
                    className={`flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 ${
                      isDup ? 'bg-yellow-50 dark:bg-yellow-900/10' : ''
                    }`}
                  >
                    <Checkbox
                      checked={selectedIndices.has(index)}
                      onCheckedChange={() => handleToggle(index)}
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium truncate">
                          {channel.name}
                        </span>
                        {isDup && (
                          <Badge
                            variant="outline"
                            className="text-yellow-600 border-yellow-400"
                          >
                            {t('channels.import.duplicate')}
                          </Badge>
                        )}
                        {(channel.credentials || channel.apiKey) && (
                          <Badge
                            variant="outline"
                            className="text-green-600 border-green-400"
                          >
                            {t('channels.import.withCredentials')}
                          </Badge>
                        )}
                      </div>
                      <div className="text-xs text-muted-foreground truncate">
                        {channel.type} · {channel.baseUrl}
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        </ResizableDialogBody>

        <ResizableDialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleImport} disabled={selectedCount === 0}>
            {t('channels.import.importCount', { count: selectedCount })}
          </Button>
        </ResizableDialogFooter>
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
