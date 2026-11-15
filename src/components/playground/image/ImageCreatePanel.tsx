import { useTranslation } from 'react-i18next'
import { Eraser, ImagePlus, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { IdPicker } from '@/components/ui/id-picker'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  useImagePlaygroundStore,
  type ImageProtocol,
} from '@/store/image-playground-store'
import { isJsonObjectOrEmpty } from '../shared/format'
import { ReferenceImages } from '../shared/reference-images'
import {
  COMMON_IMAGE_QUALITIES,
  COMMON_IMAGE_SIZES,
  supportsMultipleImages,
} from './image-protocol'

const ADVANCED_PLACEHOLDERS: Record<ImageProtocol, string> = {
  chat: '{\n  "image_config": { "aspect_ratio": "16:9" }\n}',
  responses: '{\n  "background": "transparent"\n}',
  google: '{\n  "generationConfig": { "seed": 42 }\n}',
}

export function ImageCreatePanel() {
  const { t } = useTranslation()
  const models = useImagePlaygroundStore(state => state.imageModels)
  const draft = useImagePlaygroundStore(state => state.imageDraft)
  const updateDraft = useImagePlaygroundStore(state => state.updateImageDraft)
  const isCreating = useImagePlaygroundStore(state => state.isCreating)

  const model = models.find(item => item.id === draft.modelId)
  const jsonValid = isJsonObjectOrEmpty(draft.extraJson)
  const countValid =
    Number.isInteger(draft.count) && draft.count >= 1 && draft.count <= 8
  const canSubmit =
    model !== undefined &&
    draft.prompt.trim() !== '' &&
    jsonValid &&
    countValid &&
    !isCreating
  const multiple = model ? supportsMultipleImages(model) : true
  const sizeSuggestions = model?.sizes ?? COMMON_IMAGE_SIZES.chat
  const qualitySuggestions = model?.qualities ?? COMMON_IMAGE_QUALITIES.chat
  const hasDraftContent =
    draft.prompt.trim() !== '' ||
    draft.size.trim() !== '' ||
    draft.quality.trim() !== '' ||
    draft.imageUrl.trim() !== '' ||
    draft.extraJson.trim() !== '' ||
    draft.count !== 1

  const handleSubmit = async () => {
    if (!model || !canSubmit) return
    await useImagePlaygroundStore.getState().createImageTask(model.id, {
      prompt: draft.prompt.trim(),
      size: draft.size.trim(),
      quality: draft.quality.trim(),
      count: draft.count,
      imageUrl: draft.imageUrl,
      extraJson: draft.extraJson,
    })
  }

  return (
    <div className="flex h-full flex-col">
      <div className="min-h-0 flex-1 space-y-4 overflow-auto p-4">
        <div className="grid gap-2">
          <Label htmlFor="image-prompt">
            {t('playground.image.create.prompt')}
          </Label>
          <Textarea
            id="image-prompt"
            value={draft.prompt}
            onChange={event => updateDraft({ prompt: event.target.value })}
            placeholder={t('playground.image.create.promptPlaceholder')}
            className="min-h-28 resize-y"
            onKeyDown={event => {
              if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
                event.preventDefault()
                void handleSubmit()
              }
            }}
          />
        </div>

        <div
          className={
            qualitySuggestions.length > 0
              ? 'grid grid-cols-2 gap-3'
              : 'grid gap-3'
          }
        >
          <div className="grid gap-2">
            <Label htmlFor="image-size">
              {t('playground.image.create.size')}
            </Label>
            <div className="flex gap-2">
              <Input
                id="image-size"
                value={draft.size}
                onChange={event => updateDraft({ size: event.target.value })}
                placeholder={sizeSuggestions[0] ?? ''}
                className="min-w-0 flex-1 font-mono text-xs"
              />
              <IdPicker
                label={t('playground.image.create.size')}
                value={draft.size}
                options={sizeSuggestions}
                onPick={value => updateDraft({ size: value })}
              />
            </div>
          </div>
          {qualitySuggestions.length > 0 && (
            <div className="grid gap-2">
              <Label htmlFor="image-quality">
                {t('playground.image.create.quality')}
              </Label>
              <div className="flex gap-2">
                <Input
                  id="image-quality"
                  value={draft.quality}
                  onChange={event =>
                    updateDraft({ quality: event.target.value })
                  }
                  placeholder={qualitySuggestions[0] ?? ''}
                  className="min-w-0 flex-1 font-mono text-xs"
                />
                <IdPicker
                  label={t('playground.image.create.quality')}
                  value={draft.quality}
                  options={qualitySuggestions}
                  onPick={value => updateDraft({ quality: value })}
                />
              </div>
            </div>
          )}
        </div>
        <p className="-mt-2 text-xs text-muted-foreground">
          {t('playground.image.create.optionsHint')}
        </p>

        {multiple && (
          <div className="grid gap-2">
            <Label htmlFor="image-count">
              {t('playground.image.create.count')}
            </Label>
            <Input
              id="image-count"
              type="number"
              min={1}
              max={8}
              value={Number.isFinite(draft.count) ? draft.count : 1}
              onChange={event =>
                updateDraft({
                  count: Math.min(
                    8,
                    Math.max(1, Math.trunc(Number(event.target.value) || 1))
                  ),
                })
              }
              className="w-24"
            />
            <p className="text-xs text-muted-foreground">
              {t('playground.image.create.countHint')}
            </p>
          </div>
        )}

        {model?.supportsImageInput && (
          <div className="grid gap-2">
            <Label>{t('playground.image.create.imageUrls')}</Label>
            <ReferenceImages
              value={draft.imageUrl}
              onChange={imageUrl => updateDraft({ imageUrl })}
            />
          </div>
        )}

        <details className="group rounded-md border">
          <summary className="cursor-pointer select-none px-3 py-2 text-sm font-medium">
            {t('playground.image.create.advanced')}
          </summary>
          <div className="grid gap-2 border-t p-3">
            <Textarea
              value={draft.extraJson}
              onChange={event => updateDraft({ extraJson: event.target.value })}
              placeholder={
                ADVANCED_PLACEHOLDERS[model?.protocol ?? 'chat'] ??
                ADVANCED_PLACEHOLDERS.chat
              }
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
                ? t('playground.image.create.advancedHint')
                : t('playground.image.create.invalidJson')}
            </p>
          </div>
        </details>
      </div>

      <div className="flex items-center justify-between gap-2 border-t p-3">
        <span className="text-xs text-muted-foreground">
          {t('playground.image.create.costHint')}
        </span>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            title={t('playground.image.create.clear')}
            aria-label={t('playground.image.create.clear')}
            disabled={!hasDraftContent}
            onClick={() =>
              updateDraft({
                prompt: '',
                size: '',
                quality: '',
                count: 1,
                imageUrl: '',
                extraJson: '',
              })
            }
          >
            <Eraser />
          </Button>
          <Button onClick={() => void handleSubmit()} disabled={!canSubmit}>
            {isCreating ? <Loader2 className="animate-spin" /> : <ImagePlus />}
            {isCreating
              ? t('playground.image.create.creating')
              : t('playground.image.create.submit')}
          </Button>
        </div>
      </div>
    </div>
  )
}
