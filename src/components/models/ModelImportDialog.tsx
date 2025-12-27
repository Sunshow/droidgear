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
import type { CustomModel } from '@/lib/bindings'

export type MergeStrategy = 'skip' | 'replace' | 'keep-both'

interface ModelImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  importModels: CustomModel[]
  existingModels: CustomModel[]
  onImport: (models: CustomModel[], strategy: MergeStrategy) => void
}

function isDuplicate(
  model: CustomModel,
  existingModels: CustomModel[]
): boolean {
  return existingModels.some(
    existing =>
      existing.baseUrl === model.baseUrl && existing.apiKey === model.apiKey
  )
}

function truncateUrl(url: string, maxLength = 40): string {
  if (url.length <= maxLength) return url
  return url.substring(0, maxLength - 3) + '...'
}

export function ModelImportDialog({
  open,
  onOpenChange,
  importModels,
  existingModels,
  onImport,
}: ModelImportDialogProps) {
  const { t } = useTranslation()
  const [selectedIndices, setSelectedIndices] = useState<Set<number>>(
    () => new Set(importModels.map((_, i) => i))
  )
  const [mergeStrategy, setMergeStrategy] = useState<MergeStrategy>('skip')

  const duplicateIndices = useMemo(() => {
    const indices = new Set<number>()
    importModels.forEach((model, index) => {
      if (isDuplicate(model, existingModels)) {
        indices.add(index)
      }
    })
    return indices
  }, [importModels, existingModels])

  const handleSelectAll = () => {
    setSelectedIndices(new Set(importModels.map((_, i) => i)))
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
    const selectedModels = importModels.filter((_, i) => selectedIndices.has(i))
    onImport(selectedModels, mergeStrategy)
    onOpenChange(false)
  }

  const selectedCount = selectedIndices.size
  const duplicateSelectedCount = Array.from(selectedIndices).filter(i =>
    duplicateIndices.has(i)
  ).length

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[80vh] flex flex-col overflow-hidden">
        <DialogHeader>
          <DialogTitle>{t('models.import.title')}</DialogTitle>
          <DialogDescription>
            {t('models.import.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-wrap items-center gap-2 py-2">
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">
              {t('models.import.mergeStrategy')}:
            </span>
            <Select
              value={mergeStrategy}
              onValueChange={v => setMergeStrategy(v as MergeStrategy)}
            >
              <SelectTrigger className="w-[180px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="skip">
                  {t('models.import.strategySkip')}
                </SelectItem>
                <SelectItem value="replace">
                  {t('models.import.strategyReplace')}
                </SelectItem>
                <SelectItem value="keep-both">
                  {t('models.import.strategyKeepBoth')}
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
          <div className="flex items-center gap-2 p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded-md text-sm">
            <AlertCircle className="h-4 w-4 text-yellow-600 dark:text-yellow-400" />
            <span className="text-yellow-700 dark:text-yellow-300">
              {t('models.import.duplicateWarning', {
                count: duplicateSelectedCount,
              })}
            </span>
          </div>
        )}

        <div className="min-h-0 flex-1 overflow-y-auto border rounded-md">
          <div className="p-2 space-y-1">
            {importModels.map((model, index) => {
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
                        {model.displayName || model.model}
                      </span>
                      {isDup && (
                        <Badge
                          variant="outline"
                          className="text-yellow-600 border-yellow-400"
                        >
                          {t('models.import.duplicate')}
                        </Badge>
                      )}
                    </div>
                    <div className="text-xs text-muted-foreground truncate">
                      {model.provider} · {truncateUrl(model.baseUrl)}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleImport} disabled={selectedCount === 0}>
            {t('models.import.importCount', { count: selectedCount })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
