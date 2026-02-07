import { useState, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertCircle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
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
import type {
  OpenCodeProviderConfig,
  OpenCodeCurrentConfig,
  JsonValue,
} from '@/lib/bindings'

export type ImportMergeStrategy = 'skip' | 'replace'

interface ProviderImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  importConfig: OpenCodeCurrentConfig
  existingProviderIds: string[]
  onImport: (
    providers: Record<string, OpenCodeProviderConfig>,
    auth: Record<string, JsonValue>,
    strategy: ImportMergeStrategy
  ) => void
}

export function ProviderImportDialog({
  open,
  onOpenChange,
  importConfig,
  existingProviderIds,
  onImport,
}: ProviderImportDialogProps) {
  const { t } = useTranslation()
  const providerIds = Object.keys(importConfig.providers)

  const [selectedIds, setSelectedIds] = useState<Set<string>>(
    () => new Set(providerIds)
  )
  const [mergeStrategy, setMergeStrategy] =
    useState<ImportMergeStrategy>('skip')

  // Reset selection when dialog opens with new data
  useState(() => {
    setSelectedIds(new Set(providerIds))
  })

  const conflictIds = useMemo(() => {
    return new Set(providerIds.filter(id => existingProviderIds.includes(id)))
  }, [providerIds, existingProviderIds])

  const handleSelectAll = () => {
    setSelectedIds(new Set(providerIds))
  }

  const handleDeselectAll = () => {
    setSelectedIds(new Set())
  }

  const handleToggle = (id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const handleImport = () => {
    const selectedProviders: Record<string, OpenCodeProviderConfig> = {}
    const selectedAuth: Record<string, JsonValue> = {}

    for (const id of selectedIds) {
      const config = importConfig.providers[id]
      if (config) {
        selectedProviders[id] = config
      }
      const auth = importConfig.auth[id]
      if (auth !== undefined) {
        selectedAuth[id] = auth
      }
    }

    onImport(selectedProviders, selectedAuth, mergeStrategy)
    onOpenChange(false)
  }

  const selectedCount = selectedIds.size
  const conflictSelectedCount = Array.from(selectedIds).filter(id =>
    conflictIds.has(id)
  ).length

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>{t('opencode.provider.importTitle')}</DialogTitle>
          <DialogDescription>
            {t('opencode.provider.importDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-hidden flex flex-col gap-3 py-2">
          {/* Strategy and selection controls */}
          <div className="flex flex-wrap items-center gap-2">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">
                {t('opencode.provider.mergeStrategy')}:
              </span>
              <Select
                value={mergeStrategy}
                onValueChange={v => setMergeStrategy(v as ImportMergeStrategy)}
              >
                <SelectTrigger className="w-[160px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="skip">
                    {t('opencode.provider.strategySkip')}
                  </SelectItem>
                  <SelectItem value="replace">
                    {t('opencode.provider.strategyReplace')}
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

          {/* Conflict warning */}
          {conflictSelectedCount > 0 && (
            <div className="flex items-center gap-2 p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded-md text-sm">
              <AlertCircle className="h-4 w-4 text-yellow-600 dark:text-yellow-400 shrink-0" />
              <span className="text-yellow-700 dark:text-yellow-300">
                {t('opencode.provider.conflictWarning', {
                  count: conflictSelectedCount,
                })}
              </span>
            </div>
          )}

          {/* Provider list */}
          <div className="flex-1 overflow-y-auto border rounded-md min-h-[200px]">
            <div className="p-2 space-y-1">
              {providerIds.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  {t('opencode.provider.noProvidersToImport')}
                </div>
              ) : (
                providerIds.map(id => {
                  const config = importConfig.providers[id]
                  const isConflict = conflictIds.has(id)
                  const hasAuth = importConfig.auth[id] !== undefined
                  return (
                    <div
                      key={id}
                      className={`flex items-center gap-3 p-2 rounded-md hover:bg-muted/50 ${
                        isConflict ? 'bg-yellow-50 dark:bg-yellow-900/10' : ''
                      }`}
                    >
                      <Checkbox
                        checked={selectedIds.has(id)}
                        onCheckedChange={() => handleToggle(id)}
                      />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-medium truncate">
                            {config?.name || id}
                          </span>
                          {isConflict && (
                            <Badge
                              variant="outline"
                              className="text-yellow-600 border-yellow-400"
                            >
                              {t('opencode.provider.conflict')}
                            </Badge>
                          )}
                          {hasAuth && (
                            <Badge variant="secondary" className="text-xs">
                              {t('opencode.provider.apiKeyConfigured')}
                            </Badge>
                          )}
                        </div>
                        <div className="text-xs text-muted-foreground truncate">
                          {id}
                          {config?.options?.baseURL &&
                            ` · ${config.options.baseURL}`}
                        </div>
                      </div>
                    </div>
                  )
                })
              )}
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleImport} disabled={selectedCount === 0}>
            {t('opencode.provider.importCount', { count: selectedCount })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
