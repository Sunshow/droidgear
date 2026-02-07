import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import type { OpenCodeModelConfig } from '@/lib/bindings'

interface ModelEditDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  modelId: string | null
  config: OpenCodeModelConfig | null
  existingModelIds: string[]
  onSave: (modelId: string, config: OpenCodeModelConfig) => void
}

export function ModelEditDialog({
  open,
  onOpenChange,
  modelId,
  config,
  existingModelIds,
  onSave,
}: ModelEditDialogProps) {
  const { t } = useTranslation()
  const isEditing = modelId !== null

  const [currentModelId, setCurrentModelId] = useState(modelId || '')
  const [displayName, setDisplayName] = useState(config?.name || '')
  const [contextLimit, setContextLimit] = useState(
    config?.limit?.context?.toString() || ''
  )
  const [outputLimit, setOutputLimit] = useState(
    config?.limit?.output?.toString() || ''
  )
  const [error, setError] = useState('')

  const handleSave = () => {
    const trimmedId = currentModelId.trim()
    if (!trimmedId) {
      setError(t('opencode.provider.modelIdRequired'))
      return
    }

    // Check for duplicate ID (only when adding new or changing ID)
    if (trimmedId !== modelId && existingModelIds.includes(trimmedId)) {
      setError(t('opencode.provider.modelIdDuplicate'))
      return
    }

    const newConfig: OpenCodeModelConfig = {
      name: displayName.trim() || null,
      limit:
        contextLimit || outputLimit
          ? {
              context: contextLimit ? parseInt(contextLimit, 10) : null,
              output: outputLimit ? parseInt(outputLimit, 10) : null,
            }
          : null,
    }

    onSave(trimmedId, newConfig)
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isEditing
              ? t('opencode.provider.editModel')
              : t('opencode.provider.addModel')}
          </DialogTitle>
          <DialogDescription>
            {t('opencode.provider.modelDialogDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Model ID */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.modelId')} *</Label>
            <Input
              value={currentModelId}
              onChange={e => {
                setCurrentModelId(e.target.value)
                setError('')
              }}
              placeholder={t('opencode.provider.modelIdPlaceholder')}
              disabled={isEditing}
            />
          </div>

          {/* Display Name */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.modelDisplayName')}</Label>
            <Input
              value={displayName}
              onChange={e => setDisplayName(e.target.value)}
              placeholder={t('opencode.provider.modelDisplayNamePlaceholder')}
            />
          </div>

          {/* Context Limit */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.contextLimit')}</Label>
            <Input
              type="number"
              value={contextLimit}
              onChange={e => setContextLimit(e.target.value)}
              placeholder="200000"
              min="0"
            />
          </div>

          {/* Output Limit */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.outputLimit')}</Label>
            <Input
              type="number"
              value={outputLimit}
              onChange={e => setOutputLimit(e.target.value)}
              placeholder="8192"
              min="0"
            />
          </div>

          {/* Error Message */}
          {error && <div className="text-sm text-destructive">{error}</div>}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleSave} disabled={!currentModelId.trim()}>
            {t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
