import { useMemo, useState } from 'react'
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
import {
  clampEffortToSupported,
  getSupportedEfforts,
} from '@/lib/model-registry'
import type { ReasoningEffort } from '@/lib/utils'

const FALLBACK_REASONING_EFFORTS: ReasoningEffort[] = [
  'none',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
]

interface DefaultModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  model: CustomModel | null
  currentSettings: SessionDefaultSettings | null
  onSave: (settings: SessionDefaultSettings) => Promise<void>
}

function effortsForModel(model: CustomModel | null): ReasoningEffort[] {
  if (!model?.model) return FALLBACK_REASONING_EFFORTS
  return (
    getSupportedEfforts(model.model, model.provider) ??
    FALLBACK_REASONING_EFFORTS
  )
}

interface DefaultModelFormProps {
  model: CustomModel
  currentSettings: SessionDefaultSettings | null
  onSave: (settings: SessionDefaultSettings) => Promise<void>
  onCancel: () => void
}

function DefaultModelForm({
  model,
  currentSettings,
  onSave,
  onCancel,
}: DefaultModelFormProps) {
  const { t } = useTranslation()
  const effortOptions = useMemo(() => effortsForModel(model), [model])

  const [setAsDefault, setSetAsDefault] = useState(true)
  const [setAsSpecMode, setSetAsSpecMode] = useState(true)
  const [reasoningEffort, setReasoningEffort] = useState(() =>
    clampEffortToSupported(
      currentSettings?.reasoningEffort ?? 'high',
      effortOptions
    )
  )
  const [specModeReasoningEffort, setSpecModeReasoningEffort] = useState(() =>
    clampEffortToSupported(
      currentSettings?.specModeReasoningEffort ?? 'high',
      effortOptions
    )
  )
  const [isSaving, setIsSaving] = useState(false)

  const handleSave = async () => {
    if (!model.id) return

    if (!setAsDefault && !setAsSpecMode) {
      toast.warning(t('models.defaultSettings.noSelection'))
      return
    }

    setIsSaving(true)
    try {
      const newSettings: SessionDefaultSettings = {
        model: setAsDefault ? model.id : (currentSettings?.model ?? null),
        reasoningEffort: setAsDefault
          ? clampEffortToSupported(reasoningEffort, effortOptions)
          : (currentSettings?.reasoningEffort ?? null),
        specModeModel: setAsSpecMode
          ? model.id
          : (currentSettings?.specModeModel ?? null),
        specModeReasoningEffort: setAsSpecMode
          ? clampEffortToSupported(specModeReasoningEffort, effortOptions)
          : (currentSettings?.specModeReasoningEffort ?? null),
        autonomyMode: currentSettings?.autonomyMode ?? null,
        autonomyLevel: currentSettings?.autonomyLevel ?? null,
        interactionMode: currentSettings?.interactionMode ?? null,
      }

      await onSave(newSettings)
      toast.success(t('common.saved'))
      onCancel()
    } finally {
      setIsSaving(false)
    }
  }

  const displayName = model.displayName || model.model || ''

  return (
    <>
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
          {model.model && model.model !== displayName && (
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
                  {effortOptions.map(effort => (
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
                  {effortOptions.map(effort => (
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
        <Button variant="outline" onClick={onCancel} disabled={isSaving}>
          {t('common.cancel')}
        </Button>
        <Button onClick={handleSave} disabled={isSaving}>
          {t('common.save')}
        </Button>
      </DialogFooter>
    </>
  )
}

export function DefaultModelDialog({
  open,
  onOpenChange,
  model,
  currentSettings,
  onSave,
}: DefaultModelDialogProps) {
  const formKey = model
    ? `${model.id ?? model.model}-${currentSettings?.reasoningEffort ?? ''}-${currentSettings?.specModeReasoningEffort ?? ''}`
    : 'empty'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        {open && model && (
          <DefaultModelForm
            key={formKey}
            model={model}
            currentSettings={currentSettings}
            onSave={onSave}
            onCancel={() => onOpenChange(false)}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}
