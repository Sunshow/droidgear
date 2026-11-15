import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  AlertCircle,
  ChevronLeft,
  ChevronRight,
  Loader2,
  Search,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { isApiKeyAuthChannel } from '@/lib/channel-utils'
import {
  commands,
  type Channel,
  type ChannelToken,
  type ModelInfo,
} from '@/lib/bindings'
import { useChannelStore } from '@/store/channel-store'
import type { VideoModelDraft, VideoProtocol } from '@/store/playground-store'
import { buildImportedVideoModels } from './video-protocol'

interface VideoModelImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onImported: (drafts: VideoModelDraft[]) => unknown
}

type Step = 'channel' | 'token' | 'models'

export function VideoModelImportDialog({
  open,
  onOpenChange,
  onImported,
}: VideoModelImportDialogProps) {
  const { t } = useTranslation()
  const channels = useChannelStore(state => state.channels)
  const loadChannels = useChannelStore(state => state.loadChannels)
  const keysMap = useChannelStore(state => state.keys)
  const fetchKeys = useChannelStore(state => state.fetchKeys)
  const keysFetchState = useChannelStore(state => state.keysFetchState)

  const [step, setStep] = useState<Step>('channel')
  const [channelId, setChannelId] = useState('')
  const [tokenId, setTokenId] = useState<number | null>(null)
  const [models, setModels] = useState<ModelInfo[]>([])
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [defaultProtocol, setDefaultProtocol] = useState<VideoProtocol>('sora')
  const [modelProtocols, setModelProtocols] = useState<
    Map<string, VideoProtocol>
  >(new Map())
  const [search, setSearch] = useState('')
  const [isLoadingModels, setIsLoadingModels] = useState(false)
  const [isResolvingKey, setIsResolvingKey] = useState(false)
  const [isImporting, setIsImporting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const channel: Channel | undefined = channels.find(
    item => item.id === channelId
  )
  const apiKeyAuth = channel ? isApiKeyAuthChannel(channel.type) : false
  const activeTokens = (channelId ? (keysMap[channelId] ?? []) : []).filter(
    token => token.status === 1
  )
  const tokenFetchState = channelId ? keysFetchState[channelId] : undefined

  useEffect(() => {
    if (!open) return
    setStep('channel')
    setChannelId('')
    setTokenId(null)
    setModels([])
    setSelectedIds(new Set())
    setDefaultProtocol('sora')
    setModelProtocols(new Map())
    setSearch('')
    setError(null)
    setIsLoadingModels(false)
    setIsResolvingKey(false)
    setIsImporting(false)
    if (channels.length === 0) void loadChannels()
  }, [open, channels.length, loadChannels])

  useEffect(() => {
    if (step !== 'token' || !channel) return
    if (keysMap[channel.id] !== undefined) return
    if (keysFetchState[channel.id]?.isLoading) return
    void fetchKeys(channel.id, channel.type, channel.baseUrl)
  }, [step, channel, keysMap, keysFetchState, fetchKeys])

  const loadModels = async (
    baseUrl: string,
    apiKey: string,
    platform: string | null
  ) => {
    setStep('models')
    setIsLoadingModels(true)
    setError(null)
    setModels([])
    setSelectedIds(new Set())
    setModelProtocols(new Map())
    try {
      const result = await commands.fetchModelsByApiKey(
        baseUrl,
        apiKey,
        platform
      )
      if (result.status === 'error') {
        setError(result.error)
        return
      }
      setModels(result.data)
    } catch (caught) {
      setError(String(caught))
    } finally {
      setIsLoadingModels(false)
    }
  }

  const handleChannelNext = async () => {
    if (!channel) return
    setError(null)
    if (isApiKeyAuthChannel(channel.type)) {
      setIsResolvingKey(true)
      try {
        const result = await commands.getChannelApiKey(channel.id)
        if (result.status !== 'ok' || !result.data) {
          setError(
            t('playground.video.import.missingApiKey', {
              channel: channel.name,
            })
          )
          return
        }
        setTokenId(null)
        await loadModels(channel.baseUrl, result.data, null)
      } catch (caught) {
        setError(String(caught))
      } finally {
        setIsResolvingKey(false)
      }
      return
    }
    setStep('token')
  }

  const handleTokenSelect = async (token: ChannelToken) => {
    if (!channel) return
    setTokenId(token.id)
    await loadModels(channel.baseUrl, token.key, token.platform ?? null)
  }

  const handleBack = () => {
    setError(null)
    if (step === 'models') {
      setStep(apiKeyAuth ? 'channel' : 'token')
      return
    }
    setStep('channel')
  }

  const query = search.trim().toLowerCase()
  const filteredModels =
    query === ''
      ? models
      : models.filter(
          model =>
            model.id.toLowerCase().includes(query) ||
            (model.name ?? '').toLowerCase().includes(query)
        )

  const allVisibleSelected =
    filteredModels.length > 0 &&
    filteredModels.every(model => selectedIds.has(model.id))

  const toggleModel = (id: string) => {
    setSelectedIds(previous => {
      const next = new Set(previous)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const toggleAllVisible = () => {
    setSelectedIds(previous => {
      const next = new Set(previous)
      if (allVisibleSelected) {
        filteredModels.forEach(model => next.delete(model.id))
      } else {
        filteredModels.forEach(model => next.add(model.id))
      }
      return next
    })
  }

  const handleImport = async () => {
    if (!channel || selectedIds.size === 0) return
    setIsImporting(true)
    try {
      const protocols = new Map<string, VideoProtocol>()
      selectedIds.forEach(id => {
        protocols.set(id, modelProtocols.get(id) ?? defaultProtocol)
      })
      const drafts = buildImportedVideoModels({
        models,
        selectedIds,
        channelId: channel.id,
        tokenId,
        protocols,
      })
      await onImported(drafts)
      onOpenChange(false)
    } catch (caught) {
      setError(String(caught))
    } finally {
      setIsImporting(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t('playground.video.import.title')}</DialogTitle>
          <DialogDescription>
            {t('playground.video.import.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <span
            className={step === 'channel' ? 'font-medium text-foreground' : ''}
          >
            1. {t('playground.video.import.stepChannel')}
          </span>
          {!apiKeyAuth && (
            <>
              <ChevronRight className="size-3" />
              <span
                className={
                  step === 'token' ? 'font-medium text-foreground' : ''
                }
              >
                2. {t('playground.video.import.stepToken')}
              </span>
            </>
          )}
          <ChevronRight className="size-3" />
          <span
            className={step === 'models' ? 'font-medium text-foreground' : ''}
          >
            {apiKeyAuth ? 2 : 3}. {t('playground.video.import.stepModels')}
          </span>
        </div>

        {error && (
          <div className="flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive">
            <AlertCircle className="mt-0.5 size-4 shrink-0" />
            <span className="min-w-0 break-all">{error}</span>
          </div>
        )}

        {step === 'channel' && (
          <div className="grid gap-3">
            <Label>{t('playground.video.import.channel')}</Label>
            <Select
              value={channelId}
              onValueChange={value => {
                setChannelId(value)
                setTokenId(null)
                setError(null)
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
                {channels
                  .filter(item => item.enabled)
                  .map(item => (
                    <SelectItem key={item.id} value={item.id}>
                      {item.name}
                      <span className="text-xs text-muted-foreground">
                        {item.baseUrl}
                      </span>
                    </SelectItem>
                  ))}
              </SelectContent>
            </Select>
            {channels.filter(item => item.enabled).length === 0 && (
              <p className="text-xs text-muted-foreground">
                {t('playground.video.modelDialog.noChannels')}
              </p>
            )}
            <p className="text-xs text-muted-foreground">
              {t('playground.video.import.channelHint')}
            </p>
          </div>
        )}

        {step === 'token' && (
          <div className="grid gap-3">
            <p className="text-sm text-muted-foreground">
              {t('playground.video.import.tokenHint')}
            </p>
            {tokenFetchState?.isLoading && (
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Loader2 className="size-4 animate-spin" />
                {t('common.loading')}
              </div>
            )}
            {tokenFetchState?.error && (
              <p className="text-sm text-destructive">
                {tokenFetchState.error}
              </p>
            )}
            {!tokenFetchState?.isLoading &&
              !tokenFetchState?.error &&
              activeTokens.length === 0 && (
                <p className="text-sm text-muted-foreground">
                  {t('playground.video.import.noTokens')}
                </p>
              )}
            <div className="max-h-72 divide-y overflow-auto rounded-md border">
              {activeTokens.map(token => (
                <button
                  key={token.id}
                  type="button"
                  className="flex w-full items-center gap-3 p-3 text-left hover:bg-muted/50"
                  onClick={() => void handleTokenSelect(token)}
                >
                  <div className="min-w-0 flex-1">
                    <span className="block truncate font-medium">
                      {token.name}
                    </span>
                    <p className="truncate text-xs text-muted-foreground">
                      {[
                        token.groupName,
                        token.platform,
                        token.unlimitedQuota
                          ? t('playground.video.import.unlimitedQuota')
                          : undefined,
                      ]
                        .filter(Boolean)
                        .join(' · ') || `#${token.id}`}
                    </p>
                  </div>
                  <ChevronRight className="size-4 shrink-0 text-muted-foreground" />
                </button>
              ))}
            </div>
          </div>
        )}

        {step === 'models' && (
          <div className="grid gap-3">
            <div className="flex flex-wrap items-end gap-3">
              <div className="grid gap-1">
                <Label>{t('playground.video.import.protocol')}</Label>
                <Select
                  value={defaultProtocol}
                  onValueChange={value => {
                    setDefaultProtocol(value as VideoProtocol)
                    setModelProtocols(new Map())
                  }}
                >
                  <SelectTrigger className="w-56">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="sora">
                      {t('playground.video.modelDialog.protocolSora')}
                    </SelectItem>
                    <SelectItem value="new-api">
                      {t('playground.video.modelDialog.protocolNewApi')}
                    </SelectItem>
                    <SelectItem value="doubao">
                      {t('playground.video.modelDialog.protocolDoubao')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="relative min-w-48 flex-1">
                <Search className="pointer-events-none absolute start-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={search}
                  onChange={event => setSearch(event.target.value)}
                  placeholder={t('playground.video.import.searchPlaceholder')}
                  className="ps-8"
                />
              </div>
            </div>
            <p className="text-xs text-muted-foreground">
              {t('playground.video.import.protocolHint')}
            </p>

            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <button
                type="button"
                className="hover:text-foreground"
                onClick={toggleAllVisible}
                disabled={filteredModels.length === 0}
              >
                {allVisibleSelected
                  ? t('playground.video.import.clearAll')
                  : t('common.selectAll')}
              </button>
              <span>
                {t('playground.video.import.selectedCount', {
                  count: selectedIds.size,
                })}
              </span>
            </div>

            <div className="max-h-72 overflow-auto rounded-md border">
              {isLoadingModels ? (
                <div className="flex items-center justify-center gap-2 p-8 text-sm text-muted-foreground">
                  <Loader2 className="size-4 animate-spin" />
                  {t('common.loading')}
                </div>
              ) : filteredModels.length === 0 ? (
                <p className="p-8 text-center text-sm text-muted-foreground">
                  {models.length === 0
                    ? t('playground.video.import.empty')
                    : t('playground.video.import.noMatch')}
                </p>
              ) : (
                filteredModels.map(model => (
                  <div
                    key={model.id}
                    className="flex items-start gap-3 border-b p-3 last:border-b-0 hover:bg-muted/50"
                  >
                    <Checkbox
                      id={`video-import-${model.id}`}
                      checked={selectedIds.has(model.id)}
                      onCheckedChange={() => toggleModel(model.id)}
                    />
                    <Label
                      htmlFor={`video-import-${model.id}`}
                      className="min-w-0 flex-1 cursor-pointer"
                    >
                      <span className="block truncate font-mono text-sm">
                        {model.id}
                      </span>
                      {model.name && model.name !== model.id && (
                        <span className="block truncate text-xs text-muted-foreground">
                          {model.name}
                        </span>
                      )}
                    </Label>
                    <Select
                      value={modelProtocols.get(model.id) ?? defaultProtocol}
                      onValueChange={value =>
                        setModelProtocols(previous => {
                          const next = new Map(previous)
                          next.set(model.id, value as VideoProtocol)
                          return next
                        })
                      }
                    >
                      <SelectTrigger
                        className="h-8 w-44 shrink-0"
                        aria-label={t('playground.video.import.protocol')}
                      >
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="sora">
                          {t('playground.video.modelDialog.protocolSora')}
                        </SelectItem>
                        <SelectItem value="new-api">
                          {t('playground.video.modelDialog.protocolNewApi')}
                        </SelectItem>
                        <SelectItem value="doubao">
                          {t('playground.video.modelDialog.protocolDoubao')}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        <div className="flex items-center justify-between gap-2">
          <div>
            {step !== 'channel' && (
              <Button variant="ghost" onClick={handleBack}>
                <ChevronLeft />
                {t('common.back')}
              </Button>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            {step === 'channel' && (
              <Button
                onClick={() => void handleChannelNext()}
                disabled={!channel || isResolvingKey}
              >
                {isResolvingKey && <Loader2 className="animate-spin" />}
                {t('common.next')}
              </Button>
            )}
            {step === 'models' && (
              <Button
                onClick={() => void handleImport()}
                disabled={
                  selectedIds.size === 0 || isLoadingModels || isImporting
                }
              >
                {isImporting && <Loader2 className="animate-spin" />}
                {t('playground.video.import.importCount', {
                  count: selectedIds.size,
                })}
              </Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
