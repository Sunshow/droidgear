import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, AlertCircle, ChevronRight, ChevronLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { useChannelStore } from '@/store/channel-store'
import {
  isApiKeyAuthChannel,
  isOpenAICompatiblePlatform,
} from '@/lib/channel-utils'
import { normalizeBaseUrl } from '@/lib/sub2api-platform'
import {
  commands,
  type Channel,
  type ChannelToken,
  type ModelInfo,
} from '@/lib/bindings'

interface ImportResult {
  baseUrl: string
  apiKey: string
  provider: string
  defaultModel?: string
}

interface ImportFromChannelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onImported: (result: ImportResult) => void
}

type Step = 'channel' | 'token' | 'model'

const channelTypeI18nKeys: Record<string, string> = {
  'new-api': 'channels.typeNewApi',
  'sub-2-api': 'channels.typeSub2Api',
  'cli-proxy-api': 'channels.typeCliProxyApi',
  ollama: 'channels.typeOllama',
  general: 'channels.typeGeneral',
}

export function ImportFromChannelDialog({
  open,
  onOpenChange,
  onImported,
}: ImportFromChannelDialogProps) {
  const { t } = useTranslation()
  const channels = useChannelStore(state => state.channels)
  const loadChannels = useChannelStore(state => state.loadChannels)
  const keysMap = useChannelStore(state => state.keys)
  const fetchKeys = useChannelStore(state => state.fetchKeys)
  const keysFetchState = useChannelStore(state => state.keysFetchState)

  const [step, setStep] = useState<Step>('channel')
  const [selectedChannelId, setSelectedChannelId] = useState<string>('')
  const [resolvedApiKey, setResolvedApiKey] = useState<string>('')
  const [resolvedBaseUrl, setResolvedBaseUrl] = useState<string>('')
  const [resolvedProvider, setResolvedProvider] = useState<string>('')

  const [models, setModels] = useState<ModelInfo[]>([])
  const [selectedModel, setSelectedModel] = useState<string>('')
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [modelError, setModelError] = useState<string | null>(null)
  const [isResolvingKey, setIsResolvingKey] = useState(false)
  const [resolveError, setResolveError] = useState<string | null>(null)

  const selectedChannel: Channel | undefined = channels.find(
    c => c.id === selectedChannelId
  )
  const tokens: ChannelToken[] = keysMap[selectedChannelId] ?? []
  const tokenFetchState = keysFetchState[selectedChannelId]

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (open) {
      setStep('channel')
      setSelectedChannelId('')
      setResolvedApiKey('')
      setResolvedBaseUrl('')
      setResolvedProvider('')
      setModels([])
      setSelectedModel('')
      setModelError(null)
      setResolveError(null)
      loadChannels()
    }
  }, [open, loadChannels])

  // Fetch tokens when channel changes (for new-api / sub-2-api)
  useEffect(() => {
    if (
      selectedChannelId &&
      selectedChannel &&
      !isApiKeyAuthChannel(selectedChannel.type) &&
      keysMap[selectedChannelId] === undefined
    ) {
      fetchKeys(
        selectedChannelId,
        selectedChannel.type,
        selectedChannel.baseUrl
      )
    }
  }, [selectedChannelId, selectedChannel, keysMap, fetchKeys])

  // Hermes always uses "custom" as provider regardless of channel type or model
  const inferProvider = (_modelId: string, _platform?: string | null): string =>
    'custom'

  /** Append /v1 to baseUrl unless it already ends with it */
  const withV1 = (url: string) => normalizeBaseUrl(url, '/v1')

  const handleChannelNext = async () => {
    if (!selectedChannel) return

    setResolveError(null)

    if (isApiKeyAuthChannel(selectedChannel.type)) {
      // general / ollama / cli-proxy-api: read stored API key, skip token step
      setIsResolvingKey(true)
      try {
        const result = await commands.getChannelApiKey(selectedChannel.id)
        if (result.status !== 'ok' || !result.data) {
          setResolveError(
            t('hermes.model.importDialog.noApiKey', {
              channel: selectedChannel.name,
            })
          )
          setIsResolvingKey(false)
          return
        }
        setResolvedApiKey(result.data)
        setResolvedBaseUrl(withV1(selectedChannel.baseUrl))
        setResolvedProvider('custom')
        setIsResolvingKey(false)
        // Skip token step, go straight to model step
        await fetchModelsForKey(selectedChannel.baseUrl, result.data, null)
        setStep('model')
      } catch (e) {
        setResolveError(String(e))
        setIsResolvingKey(false)
      }
    } else {
      // new-api / sub-2-api: show token picker
      setStep('token')
    }
  }

  const handleTokenSelect = async (token: ChannelToken) => {
    const apiKey = token.key
    const rawBaseUrl = selectedChannel?.baseUrl ?? ''
    const provider = inferProvider(token.name, token.platform)

    setResolvedApiKey(apiKey)
    setResolvedBaseUrl(withV1(rawBaseUrl))
    setResolvedProvider(provider)

    await fetchModelsForKey(rawBaseUrl, apiKey, token.platform)
    setStep('model')
  }

  const fetchModelsForKey = async (
    baseUrl: string,
    apiKey: string,
    platform: string | null | undefined
  ) => {
    setIsFetchingModels(true)
    setModelError(null)
    setModels([])
    setSelectedModel('')
    try {
      const result = await commands.fetchModelsByApiKey(
        baseUrl,
        apiKey,
        platform ?? null
      )
      if (result.status === 'ok') {
        setModels(result.data)
      } else {
        setModelError(result.error)
      }
    } catch (e) {
      setModelError(String(e))
    } finally {
      setIsFetchingModels(false)
    }
  }

  const handleImport = () => {
    onImported({
      baseUrl: resolvedBaseUrl,
      apiKey: resolvedApiKey,
      provider: resolvedProvider,
      defaultModel: selectedModel || undefined,
    })
    onOpenChange(false)
  }

  const handleBack = () => {
    if (step === 'model') {
      if (selectedChannel && isApiKeyAuthChannel(selectedChannel.type)) {
        setStep('channel')
      } else {
        setStep('token')
      }
    } else if (step === 'token') {
      setStep('channel')
    }
  }

  const enabledChannels = channels.filter(c => c.enabled)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[560px]">
        <DialogHeader>
          <DialogTitle>{t('hermes.model.importDialog.title')}</DialogTitle>
          <DialogDescription>
            {step === 'channel' && t('hermes.model.importDialog.selectChannel')}
            {step === 'token' && t('hermes.model.importDialog.selectToken')}
            {step === 'model' && t('hermes.model.importDialog.selectModel')}
          </DialogDescription>
        </DialogHeader>

        {/* Step 1: Select Channel */}
        {step === 'channel' && (
          <div className="space-y-4 py-2">
            {enabledChannels.length === 0 ? (
              <p className="text-sm text-muted-foreground text-center py-4">
                {t('hermes.model.importDialog.noChannels')}
              </p>
            ) : (
              <>
                <div className="space-y-2">
                  <Label>{t('hermes.model.importDialog.channel')}</Label>
                  <Select
                    value={selectedChannelId}
                    onValueChange={setSelectedChannelId}
                  >
                    <SelectTrigger>
                      <SelectValue
                        placeholder={t(
                          'hermes.model.importDialog.channelPlaceholder'
                        )}
                      />
                    </SelectTrigger>
                    <SelectContent>
                      {enabledChannels.map(ch => (
                        <SelectItem key={ch.id} value={ch.id}>
                          <span className="font-medium">{ch.name}</span>
                          <span className="ml-2 text-muted-foreground text-xs">
                            {t(channelTypeI18nKeys[ch.type] ?? ch.type)}
                          </span>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {selectedChannel && (
                  <div className="text-sm text-muted-foreground rounded-md border px-3 py-2 bg-muted/40">
                    {selectedChannel.baseUrl}
                  </div>
                )}

                {resolveError && (
                  <div className="flex items-center gap-2 text-sm text-destructive">
                    <AlertCircle className="h-4 w-4 shrink-0" />
                    <span>{resolveError}</span>
                  </div>
                )}
              </>
            )}
          </div>
        )}

        {/* Step 2: Select Token (new-api / sub-2-api) */}
        {step === 'token' && selectedChannel && (
          <div className="space-y-3 py-2">
            {tokenFetchState?.isLoading ? (
              <div className="flex items-center justify-center py-6 gap-2">
                <Loader2 className="h-5 w-5 animate-spin" />
                <span className="text-sm text-muted-foreground">
                  {t('hermes.model.importDialog.loadingTokens')}
                </span>
              </div>
            ) : tokenFetchState?.error ? (
              <div className="flex items-center gap-2 text-sm text-destructive py-2">
                <AlertCircle className="h-4 w-4 shrink-0" />
                <span>{tokenFetchState.error}</span>
              </div>
            ) : tokens.length === 0 ? (
              <p className="text-sm text-muted-foreground text-center py-4">
                {t('hermes.model.importDialog.noTokens')}
              </p>
            ) : (
              <div className="border rounded-md max-h-64 overflow-y-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{t('common.name')}</TableHead>
                      <TableHead>{t('keys.platform')}</TableHead>
                      <TableHead className="w-[80px]" />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {tokens
                      .filter(
                        tk =>
                          tk.status === 1 &&
                          isOpenAICompatiblePlatform(tk.platform)
                      )
                      .map(tk => (
                        <TableRow key={tk.id}>
                          <TableCell className="font-medium">
                            {tk.name}
                          </TableCell>
                          <TableCell>
                            {tk.platform ? (
                              <Badge variant="outline">{tk.platform}</Badge>
                            ) : (
                              <span className="text-muted-foreground">—</span>
                            )}
                          </TableCell>
                          <TableCell>
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => handleTokenSelect(tk)}
                            >
                              {t('common.select')}
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                  </TableBody>
                </Table>
              </div>
            )}
          </div>
        )}

        {/* Step 3: Select Default Model */}
        {step === 'model' && (
          <div className="space-y-4 py-2">
            {isFetchingModels ? (
              <div className="flex items-center justify-center py-6 gap-2">
                <Loader2 className="h-5 w-5 animate-spin" />
                <span className="text-sm text-muted-foreground">
                  {t('models.fetchingModels')}
                </span>
              </div>
            ) : (
              <>
                {modelError && (
                  <div className="flex items-center gap-2 text-sm text-destructive">
                    <AlertCircle className="h-4 w-4 shrink-0" />
                    <span>{modelError}</span>
                  </div>
                )}

                <div className="space-y-2">
                  <Label>{t('hermes.model.importDialog.defaultModel')}</Label>
                  {models.length > 0 ? (
                    <Select
                      value={selectedModel}
                      onValueChange={setSelectedModel}
                    >
                      <SelectTrigger>
                        <SelectValue
                          placeholder={t(
                            'hermes.model.importDialog.modelPlaceholder'
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent className="max-h-60">
                        {models.map(m => (
                          <SelectItem key={m.id} value={m.id}>
                            {m.id}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  ) : (
                    <Input
                      value={selectedModel}
                      onChange={e => setSelectedModel(e.target.value)}
                      placeholder={t('hermes.model.defaultPlaceholder')}
                    />
                  )}
                  <p className="text-xs text-muted-foreground">
                    {t('hermes.model.importDialog.skipModelHint')}
                  </p>
                </div>

                <div className="rounded-md border px-3 py-2 bg-muted/40 space-y-1 text-sm">
                  <div className="flex gap-2">
                    <span className="text-muted-foreground w-20 shrink-0">
                      {t('hermes.model.baseUrl')}
                    </span>
                    <span className="font-mono text-xs break-all">
                      {resolvedBaseUrl}
                    </span>
                  </div>
                  <div className="flex gap-2">
                    <span className="text-muted-foreground w-20 shrink-0">
                      {t('hermes.model.provider')}
                    </span>
                    <span>{resolvedProvider}</span>
                  </div>
                  <div className="flex gap-2">
                    <span className="text-muted-foreground w-20 shrink-0">
                      {t('hermes.model.apiKey')}
                    </span>
                    <span className="font-mono text-xs">
                      {'•'.repeat(Math.min(resolvedApiKey.length, 16))}
                    </span>
                  </div>
                </div>
              </>
            )}
          </div>
        )}

        <DialogFooter className="gap-2">
          {step !== 'channel' && (
            <Button variant="outline" onClick={handleBack}>
              <ChevronLeft className="h-4 w-4 mr-1" />
              {t('common.back')}
            </Button>
          )}
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>

          {step === 'channel' && (
            <Button
              onClick={handleChannelNext}
              disabled={!selectedChannelId || isResolvingKey}
            >
              {isResolvingKey ? (
                <Loader2 className="h-4 w-4 animate-spin mr-2" />
              ) : (
                <ChevronRight className="h-4 w-4 mr-1" />
              )}
              {t('common.next')}
            </Button>
          )}

          {step === 'model' && !isFetchingModels && (
            <Button onClick={handleImport}>
              {t('hermes.model.importDialog.import')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
