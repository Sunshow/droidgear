import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Download, Loader2 } from 'lucide-react'
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
import type {
  VideoModel,
  VideoModelDraft,
  VideoProtocol,
} from '@/store/playground-store'
import { parseList } from './video-format'
import { VideoModelImportDialog } from './VideoModelImportDialog'
import { videoProtocolDefaults } from './video-protocol'

interface VideoModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Present when editing; absent when adding */
  model?: VideoModel
  onSave: (draft: VideoModelDraft) => unknown
  onSaveMany: (drafts: VideoModelDraft[]) => unknown
}

type SourceKind = 'channel' | 'custom'

function VideoModelForm({
  model,
  onSave,
  onSaveMany,
  onCancel,
}: {
  model?: VideoModel
  onSave: (draft: VideoModelDraft) => unknown
  onSaveMany: (drafts: VideoModelDraft[]) => unknown
  onCancel: () => void
}) {
  const { t } = useTranslation()
  const channels = useChannelStore(state => state.channels)
  const loadChannels = useChannelStore(state => state.loadChannels)

  const initialProtocol = model?.protocol ?? 'new-api-videos'
  const initialDefaults = videoProtocolDefaults(initialProtocol)

  const [name, setName] = useState(model?.name ?? '')
  const [modelId, setModelId] = useState(model?.modelId ?? '')
  const [protocol, setProtocol] = useState<VideoProtocol>(initialProtocol)
  const [sourceKind, setSourceKind] = useState<SourceKind>(
    model?.source.kind ?? 'channel'
  )
  const [channelId, setChannelId] = useState(
    model?.source.kind === 'channel' ? model.source.channelId : ''
  )
  const [tokenId, setTokenId] = useState<number | null>(
    model?.source.kind === 'channel' ? model.source.tokenId : null
  )
  const [baseUrl, setBaseUrl] = useState(
    model?.source.kind === 'custom' ? model.source.baseUrl : ''
  )
  const [apiKey, setApiKey] = useState(
    model?.source.kind === 'custom' ? model.source.apiKey : ''
  )
  const [durations, setDurations] = useState(
    model ? model.durations.join(', ') : initialDefaults.durations.join(', ')
  )
  const [resolutions, setResolutions] = useState(
    model
      ? model.resolutions.join(', ')
      : initialDefaults.resolutions.join(', ')
  )
  const [supportsImageInput, setSupportsImageInput] = useState(
    model?.supportsImageInput ?? false
  )
  const [imageInputTouched, setImageInputTouched] = useState(Boolean(model))
  const [importOpen, setImportOpen] = useState(false)
  const [isSaving, setIsSaving] = useState(false)

  useEffect(() => {
    if (channels.length === 0) void loadChannels()
  }, [channels.length, loadChannels])

  const enabledChannels = channels.filter(channel => channel.enabled)
  const sourceValid =
    sourceKind === 'channel' ? channelId !== '' : baseUrl.trim() !== ''
  const isValid = name.trim() !== '' && modelId.trim() !== '' && sourceValid

  const handleProtocolChange = (next: VideoProtocol) => {
    const previousDefaults = videoProtocolDefaults(protocol)
    const nextDefaults = videoProtocolDefaults(next)
    if (durations === previousDefaults.durations.join(', ')) {
      setDurations(nextDefaults.durations.join(', '))
    }
    if (resolutions === previousDefaults.resolutions.join(', ')) {
      setResolutions(nextDefaults.resolutions.join(', '))
    }
    if (!imageInputTouched) {
      setSupportsImageInput(next === 'doubao')
    }
    setProtocol(next)
  }

  const handleSave = async () => {
    const draft: VideoModelDraft = {
      name: name.trim(),
      modelId: modelId.trim(),
      protocol,
      source:
        sourceKind === 'channel'
          ? { kind: 'channel', channelId, tokenId }
          : { kind: 'custom', baseUrl: baseUrl.trim(), apiKey: apiKey.trim() },
      durations: parseList(durations)
        .map(Number)
        .filter(number => Number.isFinite(number) && number > 0),
      resolutions: parseList(resolutions),
      supportsImageInput,
    }
    setIsSaving(true)
    try {
      await onSave(draft)
      onCancel()
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <>
      <ResizableDialogBody>
        <div className="grid gap-4">
          {!model && (
            <div className="flex items-center justify-between gap-3 rounded-md border bg-muted/40 p-3">
              <div className="min-w-0">
                <p className="text-sm font-medium">
                  {t('playground.video.modelDialog.importTitle')}
                </p>
                <p className="text-xs text-muted-foreground">
                  {t('playground.video.modelDialog.importHint')}
                </p>
              </div>
              <Button
                variant="outline"
                size="sm"
                className="shrink-0"
                onClick={() => setImportOpen(true)}
              >
                <Download />
                {t('playground.video.modelDialog.importAction')}
              </Button>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <div className="grid gap-2">
              <Label htmlFor="vm-name">
                {t('playground.video.modelDialog.name')}
              </Label>
              <Input
                id="vm-name"
                value={name}
                onChange={event => setName(event.target.value)}
                placeholder="Seedance 2.0"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="vm-model-id">
                {t('playground.video.modelDialog.modelId')}
              </Label>
              <Input
                id="vm-model-id"
                value={modelId}
                onChange={event => setModelId(event.target.value)}
                placeholder="doubao-seedance-2-0-260128"
                className="font-mono"
              />
            </div>
          </div>

          <div className="grid gap-2">
            <Label>{t('playground.video.modelDialog.protocol')}</Label>
            <Select
              value={protocol}
              onValueChange={value =>
                handleProtocolChange(value as VideoProtocol)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="new-api-videos">
                  {t('playground.video.modelDialog.protocolNewApiVideos')}
                </SelectItem>
                <SelectItem value="doubao">
                  {t('playground.video.modelDialog.protocolDoubao')}
                </SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {t('playground.video.modelDialog.protocolHint')}
            </p>
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
              <Select
                value={channelId}
                onValueChange={value => {
                  setChannelId(value)
                  setTokenId(null)
                }}
              >
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
              {enabledChannels.length === 0 ? (
                <p className="text-xs text-muted-foreground">
                  {t('playground.video.modelDialog.noChannels')}
                </p>
              ) : channelId ? (
                <p className="text-xs text-muted-foreground">
                  {t('playground.video.modelDialog.channelTokenHint')}
                </p>
              ) : null}
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
                  onChange={event => setBaseUrl(event.target.value)}
                  placeholder="https://ark.cn-beijing.volces.com"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="vm-api-key">
                  {t('playground.video.modelDialog.apiKey')}
                </Label>
                <SecretInput
                  id="vm-api-key"
                  value={apiKey}
                  onChange={event => setApiKey(event.target.value)}
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
                onChange={event => setDurations(event.target.value)}
                placeholder={videoProtocolDefaults(protocol).durations.join(
                  ', '
                )}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="vm-resolutions">
                {t('playground.video.modelDialog.resolutions')}
              </Label>
              <Input
                id="vm-resolutions"
                value={resolutions}
                onChange={event => setResolutions(event.target.value)}
                placeholder={videoProtocolDefaults(protocol).resolutions.join(
                  ', '
                )}
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
              onCheckedChange={checked => {
                setImageInputTouched(true)
                setSupportsImageInput(checked)
              }}
            />
          </div>
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button
          onClick={() => void handleSave()}
          disabled={!isValid || isSaving}
        >
          {isSaving && <Loader2 className="animate-spin" />}
          {model ? t('common.save') : t('common.add')}
        </Button>
      </ResizableDialogFooter>

      <VideoModelImportDialog
        open={importOpen}
        onOpenChange={setImportOpen}
        onImported={async drafts => {
          await onSaveMany(drafts)
          onCancel()
        }}
      />
    </>
  )
}

export function VideoModelDialog({
  open,
  onOpenChange,
  model,
  onSave,
  onSaveMany,
}: VideoModelDialogProps) {
  const { t } = useTranslation()

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={620}
        defaultHeight={660}
        minWidth={520}
        minHeight={480}
        onCloseAutoFocus={event => event.preventDefault()}
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
            onSave={onSave}
            onSaveMany={onSaveMany}
            onCancel={() => onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
