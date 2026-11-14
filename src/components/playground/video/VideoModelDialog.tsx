import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { SecretInput } from '@/components/ui/secret-input'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  ResizableDialog,
  ResizableDialogBody,
  ResizableDialogContent,
  ResizableDialogFooter,
  ResizableDialogHeader,
  ResizableDialogTitle,
} from '@/components/ui/resizable-dialog'
import { useChannelStore } from '@/store/channel-store'
import type { VideoModel, VideoModelDraft } from '@/store/playground-store'
import { parseList } from './video-format'

interface VideoModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Present when editing; absent when adding */
  model?: VideoModel
  onSave: (draft: VideoModelDraft) => void
}

type SourceKind = 'channel' | 'custom'

function VideoModelForm({
  model,
  onSave,
  onCancel,
}: {
  model?: VideoModel
  onSave: (draft: VideoModelDraft) => void
  onCancel: () => void
}) {
  const { t } = useTranslation()
  const channels = useChannelStore(state => state.channels)
  const loadChannels = useChannelStore(state => state.loadChannels)

  const [name, setName] = useState(model?.name ?? '')
  const [modelId, setModelId] = useState(model?.modelId ?? '')
  const [sourceKind, setSourceKind] = useState<SourceKind>(
    model?.source.kind ?? 'channel'
  )
  const [channelId, setChannelId] = useState(
    model?.source.kind === 'channel' ? model.source.channelId : ''
  )
  const [baseUrl, setBaseUrl] = useState(
    model?.source.kind === 'custom' ? model.source.baseUrl : ''
  )
  const [apiKey, setApiKey] = useState(
    model?.source.kind === 'custom' ? model.source.apiKey : ''
  )
  const [durations, setDurations] = useState(
    model?.durations.join(', ') ?? '5, 10'
  )
  const [resolutions, setResolutions] = useState(
    model?.resolutions.join(', ') ?? '1280x720, 720x1280'
  )
  const [supportsImageInput, setSupportsImageInput] = useState(
    model?.supportsImageInput ?? false
  )

  useEffect(() => {
    if (channels.length === 0) void loadChannels()
  }, [channels.length, loadChannels])

  const enabledChannels = channels.filter(c => c.enabled)
  const sourceValid =
    sourceKind === 'channel' ? channelId !== '' : baseUrl.trim() !== ''
  const isValid = name.trim() !== '' && modelId.trim() !== '' && sourceValid

  const handleSave = () => {
    onSave({
      name: name.trim(),
      modelId: modelId.trim(),
      source:
        sourceKind === 'channel'
          ? { kind: 'channel', channelId }
          : { kind: 'custom', baseUrl: baseUrl.trim(), apiKey: apiKey.trim() },
      durations: parseList(durations)
        .map(Number)
        .filter(n => Number.isFinite(n) && n > 0),
      resolutions: parseList(resolutions),
      supportsImageInput,
    })
  }

  return (
    <>
      <ResizableDialogBody>
        <div className="grid gap-4">
          <div className="grid grid-cols-2 gap-3">
            <div className="grid gap-2">
              <Label htmlFor="vm-name">
                {t('playground.video.modelDialog.name')}
              </Label>
              <Input
                id="vm-name"
                value={name}
                onChange={e => setName(e.target.value)}
                placeholder="Kling v2"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="vm-model-id">
                {t('playground.video.modelDialog.modelId')}
              </Label>
              <Input
                id="vm-model-id"
                value={modelId}
                onChange={e => setModelId(e.target.value)}
                placeholder="kling-v2-master"
                className="font-mono"
              />
            </div>
          </div>

          <div className="grid gap-2">
            <Label>{t('playground.video.modelDialog.source')}</Label>
            <ToggleGroup
              type="single"
              variant="outline"
              size="sm"
              value={sourceKind}
              onValueChange={value => {
                if (value) setSourceKind(value as SourceKind)
              }}
            >
              <ToggleGroupItem value="channel" className="px-4">
                {t('playground.video.modelDialog.sourceChannel')}
              </ToggleGroupItem>
              <ToggleGroupItem value="custom" className="px-4">
                {t('playground.video.modelDialog.sourceCustom')}
              </ToggleGroupItem>
            </ToggleGroup>
          </div>

          {sourceKind === 'channel' ? (
            <div className="grid gap-2">
              <Label>{t('playground.video.modelDialog.channel')}</Label>
              <Select value={channelId} onValueChange={setChannelId}>
                <SelectTrigger className="w-full">
                  <SelectValue
                    placeholder={t(
                      'playground.video.modelDialog.channelPlaceholder'
                    )}
                  />
                </SelectTrigger>
                <SelectContent>
                  {enabledChannels.map(channel => (
                    <SelectItem key={channel.id} value={channel.id}>
                      {channel.name}
                      <span className="text-xs text-muted-foreground">
                        {channel.baseUrl}
                      </span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {enabledChannels.length === 0 && (
                <p className="text-xs text-muted-foreground">
                  {t('playground.video.modelDialog.noChannels')}
                </p>
              )}
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-3">
              <div className="grid gap-2">
                <Label htmlFor="vm-base-url">
                  {t('playground.video.modelDialog.baseUrl')}
                </Label>
                <Input
                  id="vm-base-url"
                  value={baseUrl}
                  onChange={e => setBaseUrl(e.target.value)}
                  placeholder="https://api.example.com"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="vm-api-key">
                  {t('playground.video.modelDialog.apiKey')}
                </Label>
                <SecretInput
                  id="vm-api-key"
                  value={apiKey}
                  onChange={e => setApiKey(e.target.value)}
                  placeholder="sk-..."
                />
              </div>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <div className="grid gap-2">
              <Label htmlFor="vm-durations">
                {t('playground.video.modelDialog.durations')}
              </Label>
              <Input
                id="vm-durations"
                value={durations}
                onChange={e => setDurations(e.target.value)}
                placeholder="5, 10"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="vm-resolutions">
                {t('playground.video.modelDialog.resolutions')}
              </Label>
              <Input
                id="vm-resolutions"
                value={resolutions}
                onChange={e => setResolutions(e.target.value)}
                placeholder="1280x720, 720x1280"
              />
            </div>
          </div>
          <p className="-mt-2 text-xs text-muted-foreground">
            {t('playground.video.modelDialog.optionsHint')}
          </p>

          <div className="flex items-center justify-between">
            <Label htmlFor="vm-image-input">
              {t('playground.video.modelDialog.supportsImageInput')}
            </Label>
            <Switch
              id="vm-image-input"
              checked={supportsImageInput}
              onCheckedChange={setSupportsImageInput}
            />
          </div>
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button onClick={handleSave} disabled={!isValid}>
          {model ? t('common.save') : t('common.add')}
        </Button>
      </ResizableDialogFooter>
    </>
  )
}

export function VideoModelDialog({
  open,
  onOpenChange,
  model,
  onSave,
}: VideoModelDialogProps) {
  const { t } = useTranslation()

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={600}
        defaultHeight={560}
        minWidth={500}
        minHeight={420}
        onCloseAutoFocus={e => e.preventDefault()}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>
            {model
              ? t('playground.video.modelDialog.editTitle')
              : t('playground.video.modelDialog.addTitle')}
          </ResizableDialogTitle>
          <p className="text-sm text-muted-foreground">
            {t('playground.video.modelDialog.description')}
          </p>
        </ResizableDialogHeader>
        {open && (
          <VideoModelForm
            key={model?.id ?? 'new'}
            model={model}
            onSave={draft => {
              onSave(draft)
              onOpenChange(false)
            }}
            onCancel={() => onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
