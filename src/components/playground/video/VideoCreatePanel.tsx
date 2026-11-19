import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { Eraser, Loader2, Sparkles } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { IdPicker } from '@/components/ui/id-picker'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { usePlaygroundStore } from '@/store/playground-store'
import { isJsonObjectOrEmpty } from './video-format'
import { VideoReferenceImages } from './VideoReferenceImages'
import { videoDurationOptions, videoResolutionOptions } from './video-protocol'

export function VideoCreatePanel() {
  const { t } = useTranslation()
  const models = usePlaygroundStore(state => state.videoModels)
  const draft = usePlaygroundStore(state => state.videoDraft)
  const updateDraft = usePlaygroundStore(state => state.updateVideoDraft)
  const isCreating = usePlaygroundStore(state => state.isCreating)

  const model = models.find(m => m.id === draft.modelId)
  // A model's own lists win; otherwise the shared presets keep the pickers
  // usable, matching the free-input fields they sit next to.
  const durationOptions = videoDurationOptions(model?.durations ?? [])
  const resolutionOptions = videoResolutionOptions(model?.resolutions ?? [])
  const jsonValid = isJsonObjectOrEmpty(draft.extraJson)
  const canSubmit =
    model !== undefined &&
    draft.prompt.trim() !== '' &&
    jsonValid &&
    !isCreating
  const hasDraftContent =
    draft.prompt.trim() !== '' ||
    draft.duration.trim() !== '' ||
    draft.resolution.trim() !== '' ||
    draft.imageUrl.trim() !== '' ||
    draft.extraJson.trim() !== ''

  const handleSubmit = async () => {
    if (!model || !canSubmit) return
    const taskId = await usePlaygroundStore
      .getState()
      .createVideoTask(model.id, {
        prompt: draft.prompt.trim(),
        duration: draft.duration,
        resolution: draft.resolution,
        imageUrl: draft.imageUrl.trim(),
        extraJson: draft.extraJson,
      })
    if (taskId) {
      toast.success(t('playground.video.create.submitted'))
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="min-h-0 flex-1 space-y-4 overflow-auto p-4">
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
                void handleSubmit()
              }
            }}
          />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="grid gap-2">
            <Label htmlFor="video-duration">
              {t('playground.video.create.duration')}
            </Label>
            <div className="flex gap-2">
              <Input
                id="video-duration"
                value={draft.duration}
                onChange={e => updateDraft({ duration: e.target.value })}
                placeholder={model?.durations?.[0]?.toString() ?? '5'}
                inputMode="numeric"
                className="min-w-0 flex-1"
              />
              <IdPicker
                label={t('playground.video.create.duration')}
                value={draft.duration}
                options={durationOptions}
                onPick={value => updateDraft({ duration: value })}
              />
            </div>
          </div>
          <div className="grid gap-2">
            <Label htmlFor="video-resolution">
              {t('playground.video.create.resolution')}
            </Label>
            <div className="flex gap-2">
              <Input
                id="video-resolution"
                value={draft.resolution}
                onChange={e => updateDraft({ resolution: e.target.value })}
                placeholder={model?.resolutions?.[0] ?? '1280x720'}
                className="min-w-0 flex-1 font-mono text-xs"
              />
              <IdPicker
                label={t('playground.video.create.resolution')}
                value={draft.resolution}
                options={resolutionOptions}
                onPick={value => updateDraft({ resolution: value })}
              />
            </div>
          </div>
        </div>
        <p className="-mt-2 text-xs text-muted-foreground">
          {t('playground.video.create.optionsHint')}
        </p>

        {model?.supportsImageInput && (
          <div className="grid gap-2">
            <Label>{t('playground.video.create.imageUrls')}</Label>
            <VideoReferenceImages
              value={draft.imageUrl}
              onChange={imageUrl => updateDraft({ imageUrl })}
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
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            title={t('playground.video.create.clear')}
            aria-label={t('playground.video.create.clear')}
            disabled={!hasDraftContent}
            onClick={() =>
              updateDraft({
                prompt: '',
                duration: '',
                resolution: '',
                imageUrl: '',
                extraJson: '',
              })
            }
          >
            <Eraser />
          </Button>
          <Button onClick={() => void handleSubmit()} disabled={!canSubmit}>
            {isCreating ? <Loader2 className="animate-spin" /> : <Sparkles />}
            {isCreating
              ? t('playground.video.create.creating')
              : t('playground.video.create.submit')}
          </Button>
        </div>
      </div>
    </div>
  )
}
