import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { CustomModel, SessionDefaultSettings } from '@/lib/bindings'

const REASONING_EFFORTS = [
  'none',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
] as const

interface DefaultModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  model: CustomModel | null
  currentSettings: SessionDefaultSettings | null
  onSave: (settings: SessionDefaultSettings) => Promise<void>
}

export function DefaultModelDialog({
  open,
  onOpenChange,
  model,
  currentSettings,
  onSave,
}: DefaultModelDialogProps) {
  const { t } = useTranslation()
  const [setAsDefault, setSetAsDefault] = useState(true)
  const [setAsSpecMode, setSetAsSpecMode] = useState(true)
  const [reasoningEffort, setReasoningEffort] = useState('high')
  const [specModeReasoningEffort, setSpecModeReasoningEffort] = useState('high')
  const [isSaving, setIsSaving] = useState(false)

  // Reset state when dialog opens with new model
  useEffect(() => {
    if (open && model) {
      setSetAsDefault(true)
      setSetAsSpecMode(true)
      setReasoningEffort(currentSettings?.reasoningEffort ?? 'high')
      setSpecModeReasoningEffort(
        currentSettings?.specModeReasoningEffort ?? 'high'
      )
    }
  }, [open, model, currentSettings])

  const handleSave = async () => {
    if (!model?.id) return

    if (!setAsDefault && !setAsSpecMode) {
      toast.warning(t('models.defaultSettings.noSelection'))
      return
    }

    setIsSaving(true)
    try {
      const newSettings: SessionDefaultSettings = {
        model: setAsDefault ? model.id : (currentSettings?.model ?? null),
        reasoningEffort: setAsDefault
          ? reasoningEffort
          : (currentSettings?.reasoningEffort ?? null),
        specModeModel: setAsSpecMode
          ? model.id
          : (currentSettings?.specModeModel ?? null),
        specModeReasoningEffort: setAsSpecMode
          ? specModeReasoningEffort
          : (currentSettings?.specModeReasoningEffort ?? null),
        autonomyMode: currentSettings?.autonomyMode ?? null,
        autonomyLevel: currentSettings?.autonomyLevel ?? null,
        interactionMode: currentSettings?.interactionMode ?? null,
      }

      await onSave(newSettings)
      toast.success(t('common.saved'))
      onOpenChange(false)
    } finally {
      setIsSaving(false)
    }
  }

  const displayName = model?.displayName || model?.model || ''

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>{t('models.defaultSettings.title')}</DialogTitle>
          <DialogDescription>
            {t('models.defaultSettings.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5 py-2">
          {/* Target model info */}
          <div className="px-3 py-2 bg-muted rounded-md">
            <span className="text-sm font-medium">{displayName}</span>
            {model?.model && model.model !== displayName && (
              <span className="text-xs text-muted-foreground ml-2">
                {model.model}
              </span>
            )}
          </div>

          {/* Default Model Section */}
          <div className="space-y-3">
            <div className="flex items-start gap-3">
              <Checkbox
                id="set-as-default"
                checked={setAsDefault}
                onCheckedChange={checked => setSetAsDefault(checked === true)}
                className="mt-0.5"
              />
              <div className="flex-1 space-y-1">
                <Label htmlFor="set-as-default" className="cursor-pointer">
                  {t('models.defaultSettings.setAsDefault')}
                </Label>
                <p className="text-xs text-muted-foreground">
                  {t('models.defaultSettings.setAsDefaultDesc')}
                </p>
              </div>
            </div>
            {setAsDefault && (
              <div className="ml-7 space-y-1.5">
                <Label className="text-xs text-muted-foreground">
                  {t('models.defaultSettings.reasoningEffort')}
                </Label>
                <Select
                  value={reasoningEffort}
                  onValueChange={setReasoningEffort}
                >
                  <SelectTrigger className="w-full h-8 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {REASONING_EFFORTS.map(effort => (
                      <SelectItem key={effort} value={effort}>
                        {t(`models.defaultSettings.reasoningEffort.${effort}`)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}
          </div>

          {/* Spec Mode Section */}
          <div className="space-y-3">
            <div className="flex items-start gap-3">
              <Checkbox
                id="set-as-spec-mode"
                checked={setAsSpecMode}
                onCheckedChange={checked => setSetAsSpecMode(checked === true)}
                className="mt-0.5"
              />
              <div className="flex-1 space-y-1">
                <Label htmlFor="set-as-spec-mode" className="cursor-pointer">
                  {t('models.defaultSettings.setAsSpecMode')}
                </Label>
                <p className="text-xs text-muted-foreground">
                  {t('models.defaultSettings.setAsSpecModeDesc')}
                </p>
              </div>
            </div>
            {setAsSpecMode && (
              <div className="ml-7 space-y-1.5">
                <Label className="text-xs text-muted-foreground">
                  {t('models.defaultSettings.reasoningEffort')}
                </Label>
                <Select
                  value={specModeReasoningEffort}
                  onValueChange={setSpecModeReasoningEffort}
                >
                  <SelectTrigger className="w-full h-8 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {REASONING_EFFORTS.map(effort => (
                      <SelectItem key={effort} value={effort}>
                        {t(`models.defaultSettings.reasoningEffort.${effort}`)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isSaving}
          >
            {t('common.cancel')}
          </Button>
          <Button onClick={handleSave} disabled={isSaving}>
            {t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
