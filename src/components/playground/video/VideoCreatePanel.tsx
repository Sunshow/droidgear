import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { Plus, Settings2, Sparkles } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { usePlaygroundStore } from '@/store/playground-store'
import { isJsonObjectOrEmpty } from './video-format'

const AUTO = '__auto__'

interface VideoCreatePanelProps {
  onAddModel: () => void
  onManageModels: () => void
}

export function VideoCreatePanel({
  onAddModel,
  onManageModels,
}: VideoCreatePanelProps) {
  const { t } = useTranslation()
  const models = usePlaygroundStore(state => state.videoModels)
  const draft = usePlaygroundStore(state => state.videoDraft)
  const updateDraft = usePlaygroundStore(state => state.updateVideoDraft)

  const model = models.find(m => m.id === draft.modelId)
  const jsonValid = isJsonObjectOrEmpty(draft.extraJson)
  const canSubmit =
    model !== undefined && draft.prompt.trim() !== '' && jsonValid

  const handleSubmit = () => {
    if (!model || !canSubmit) return
    usePlaygroundStore.getState().createVideoTask(model.id, {
      prompt: draft.prompt.trim(),
      duration: draft.duration,
      resolution: draft.resolution,
      imageUrl: draft.imageUrl.trim(),
      extraJson: draft.extraJson,
    })
    toast.success(t('playground.video.create.submitted'))
  }

  return (
    <div className="flex h-full flex-col">
      <div className="min-h-0 flex-1 space-y-4 overflow-auto p-4">
        <div className="grid gap-2">
          <div className="flex items-center justify-between">
            <Label>{t('playground.video.create.model')}</Label>
            <Button
              variant="ghost"
              size="sm"
              className="h-6 px-2 text-xs"
              onClick={onManageModels}
            >
              <Settings2 />
              {t('playground.video.manageModels')}
            </Button>
          </div>
          {models.length === 0 ? (
            <Button
              variant="outline"
              className="justify-start"
              onClick={onAddModel}
            >
              <Plus />
              {t('playground.video.addModel')}
            </Button>
          ) : (
            <Select
              value={model ? draft.modelId : ''}
              onValueChange={value => {
                if (value === '__add__') {
                  onAddModel()
                  return
                }
                updateDraft({ modelId: value, duration: '', resolution: '' })
              }}
            >
              <SelectTrigger className="w-full">
                <SelectValue
                  placeholder={t('playground.video.create.modelPlaceholder')}
                />
              </SelectTrigger>
              <SelectContent>
                {models.map(m => (
                  <SelectItem key={m.id} value={m.id}>
                    {m.name}
                    <span className="font-mono text-xs text-muted-foreground">
                      {m.modelId}
                    </span>
                  </SelectItem>
                ))}
                <SelectItem value="__add__" className="text-primary">
                  <Plus />
                  {t('playground.video.addModel')}
                </SelectItem>
              </SelectContent>
            </Select>
          )}
        </div>

        <div className="grid gap-2">
          <Label htmlFor="video-prompt">
            {t('playground.video.create.prompt')}
          </Label>
          <Textarea
            id="video-prompt"
            value={draft.prompt}
            onChange={e => updateDraft({ prompt: e.target.value })}
            placeholder={t('playground.video.create.promptPlaceholder')}
            className="min-h-28 resize-y"
            onKeyDown={e => {
              if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                e.preventDefault()
                handleSubmit()
              }
            }}
          />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="grid gap-2">
            <Label>{t('playground.video.create.duration')}</Label>
            <Select
              value={draft.duration || AUTO}
              onValueChange={value =>
                updateDraft({ duration: value === AUTO ? '' : value })
              }
              disabled={!model}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={AUTO}>
                  {t('playground.video.create.default')}
                </SelectItem>
                {model?.durations.map(d => (
                  <SelectItem key={d} value={String(d)}>
                    {d}s
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid gap-2">
            <Label>{t('playground.video.create.resolution')}</Label>
            <Select
              value={draft.resolution || AUTO}
              onValueChange={value =>
                updateDraft({ resolution: value === AUTO ? '' : value })
              }
              disabled={!model}
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value={AUTO}>
                  {t('playground.video.create.default')}
                </SelectItem>
                {model?.resolutions.map(r => (
                  <SelectItem key={r} value={r}>
                    {r}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        {model?.supportsImageInput && (
          <div className="grid gap-2">
            <Label htmlFor="video-image">
              {t('playground.video.create.imageUrl')}
            </Label>
            <Input
              id="video-image"
              value={draft.imageUrl}
              onChange={e => updateDraft({ imageUrl: e.target.value })}
              placeholder="https://… / data:image/png;base64,…"
            />
          </div>
        )}

        <details className="group rounded-md border">
          <summary className="cursor-pointer select-none px-3 py-2 text-sm font-medium">
            {t('playground.video.create.advanced')}
          </summary>
          <div className="grid gap-2 border-t p-3">
            <Textarea
              value={draft.extraJson}
              onChange={e => updateDraft({ extraJson: e.target.value })}
              placeholder={'{\n  "seed": 42,\n  "negative_prompt": "blurry"\n}'}
              className="min-h-24 font-mono text-xs"
              aria-invalid={!jsonValid}
            />
            <p
              className={
                jsonValid
                  ? 'text-xs text-muted-foreground'
                  : 'text-xs text-destructive'
              }
            >
              {jsonValid
                ? t('playground.video.create.advancedHint')
                : t('playground.video.create.invalidJson')}
            </p>
          </div>
        </details>
      </div>

      <div className="flex items-center justify-between gap-2 border-t p-3">
        <span className="text-xs text-muted-foreground">
          {t('playground.video.create.costHint')}
        </span>
        <Button onClick={handleSubmit} disabled={!canSubmit}>
          <Sparkles />
          {t('playground.video.create.submit')}
        </Button>
      </div>
    </div>
  )
}
