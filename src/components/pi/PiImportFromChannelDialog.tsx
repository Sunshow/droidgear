import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, AlertCircle, ChevronRight, ChevronLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
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
import { isApiKeyAuthChannel } from '@/lib/channel-utils'
import { normalizeBaseUrl } from '@/lib/sub2api-platform'
import {
  commands,
  type Channel,
  type ChannelToken,
  type ModelInfo,
  type PiModel,
} from '@/lib/bindings'

export interface PiImportResult {
  providerId: string
  baseUrl: string
  apiKey: string
  api: string
  models: PiModel[]
}

interface PiImportFromChannelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onImported: (result: PiImportResult) => void
  existingProviderIds: string[]
}

type Step = 'channel' | 'token' | 'configure'

const channelTypeI18nKeys: Record<string, string> = {
  'new-api': 'channels.typeNewApi',
  'sub-2-api': 'channels.typeSub2Api',
  'cli-proxy-api': 'channels.typeCliProxyApi',
  ollama: 'channels.typeOllama',
  general: 'channels.typeGeneral',
}

function sanitizeProviderId(raw: string): string {
  return raw
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

/** Infer Pi API type from platform string */
function inferPiApiType(
  platform: string | null | undefined,
  modelId?: string
): string {
  if (platform === 'anthropic' || platform === 'claude')
    return 'anthropic-messages'
  if (platform === 'gemini') return 'google-generative-ai'
  if (platform === 'antigravity') {
    const lower = (modelId ?? '').toLowerCase()
    if (lower.startsWith('claude-')) return 'anthropic-messages'
    return 'openai-completions'
  }
  return 'openai-completions'
}

export function PiImportFromChannelDialog({
  open,
  onOpenChange,
  onImported,
  existingProviderIds,
}: PiImportFromChannelDialogProps) {
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
  const [resolvedPlatform, setResolvedPlatform] = useState<string | null>(null)
  const [providerId, setProviderId] = useState<string>('')

  const [models, setModels] = useState<ModelInfo[]>([])
  const [selectedModelIds, setSelectedModelIds] = useState<Set<string>>(
    new Set()
  )
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [modelError, setModelError] = useState<string | null>(null)
  const [isResolvingKey, setIsResolvingKey] = useState(false)
  const [resolveError, setResolveError] = useState<string | null>(null)

  const selectedChannel: Channel | undefined = channels.find(
    c => c.id === selectedChannelId
  )
  const tokens: ChannelToken[] = keysMap[selectedChannelId] ?? []
  const tokenFetchState = keysFetchState[selectedChannelId]

  // Reset state when dialog opens
  useEffect(() => {
    if (open) {
      setStep('channel')
      setSelectedChannelId('')
      setResolvedApiKey('')
      setResolvedBaseUrl('')
      setResolvedPlatform(null)
      setProviderId('')
      setModels([])
      setSelectedModelIds(new Set())
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

  const normalizeBaseUrlForPi = (
    url: string,
    platform: string | null | undefined
  ): string => {
    // Only append /v1 for OpenAI-compatible platforms
    if (!platform || platform === 'openai') return normalizeBaseUrl(url, '/v1')
    // Anthropic, Gemini, etc. use their own base URL as-is
    return url.replace(/\/+$/, '')
  }

  const computeDefaultProviderId = (
    channel: Channel,
    tokenName?: string
  ): string => {
    const base = sanitizeProviderId(
      tokenName ? `${channel.name}-${tokenName}` : channel.name
    )
    if (!base) return 'imported-provider'
    if (!existingProviderIds.includes(base)) return base
    let i = 2
    while (existingProviderIds.includes(`${base}-${i}`)) i++
    return `${base}-${i}`
  }

  const handleChannelNext = async () => {
    if (!selectedChannel) return
    setResolveError(null)

    if (isApiKeyAuthChannel(selectedChannel.type)) {
      setIsResolvingKey(true)
      try {
        const result = await commands.getChannelApiKey(selectedChannel.id)
        if (result.status !== 'ok' || !result.data) {
          setResolveError(
            t('pi.provider.importDialog.noApiKey', {
              channel: selectedChannel.name,
            })
          )
          setIsResolvingKey(false)
          return
        }
        setResolvedApiKey(result.data)
        setResolvedBaseUrl(normalizeBaseUrlForPi(selectedChannel.baseUrl, null))
        setResolvedPlatform(null)
        setProviderId(computeDefaultProviderId(selectedChannel))
        setIsResolvingKey(false)
        await fetchModelsForKey(selectedChannel.baseUrl, result.data, null)
        setStep('configure')
      } catch (e) {
        setResolveError(String(e))
        setIsResolvingKey(false)
      }
    } else {
      setStep('token')
    }
  }

  const handleTokenSelect = async (token: ChannelToken) => {
    if (!selectedChannel) return
    const apiKey = token.key
    const rawBaseUrl = selectedChannel.baseUrl

    setResolvedApiKey(apiKey)
    setResolvedBaseUrl(normalizeBaseUrlForPi(rawBaseUrl, token.platform))
    setResolvedPlatform(token.platform ?? null)
    setProviderId(computeDefaultProviderId(selectedChannel, token.name))

    await fetchModelsForKey(rawBaseUrl, apiKey, token.platform)
    setStep('configure')
  }

  const fetchModelsForKey = async (
    baseUrl: string,
    apiKey: string,
    platform: string | null | undefined
  ) => {
    setIsFetchingModels(true)
    setModelError(null)
    setModels([])
    setSelectedModelIds(new Set())
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

  const toggleModel = (id: string) => {
    setSelectedModelIds(prev => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const toggleAllModels = (checked: boolean) => {
    if (checked) {
      setSelectedModelIds(new Set(models.map(m => m.id)))
    } else {
      setSelectedModelIds(new Set())
    }
  }

  const handleImport = () => {
    if (!providerId.trim()) return
    const firstModelId =
      selectedModelIds.size > 0 ? Array.from(selectedModelIds)[0] : undefined
    const api = inferPiApiType(resolvedPlatform, firstModelId)
    const piModels: PiModel[] = Array.from(selectedModelIds).map(id => ({
      id,
      name: null,
      api: null,
      reasoning: false,
      input: ['text'],
      contextWindow: 128000,
      maxTokens: 16384,
      cost: null,
      compat: null,
    }))
    onImported({
      providerId: providerId.trim(),
      baseUrl: resolvedBaseUrl,
      apiKey: resolvedApiKey,
      api,
      models: piModels,
    })
    onOpenChange(false)
  }

  const handleBack = () => {
    if (step === 'configure') {
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

  const trimmedProviderId = providerId.trim()
  const providerIdConflict =
    !!trimmedProviderId && existingProviderIds.includes(trimmedProviderId)
  const canImport = !!trimmedProviderId && !providerIdConflict

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px] max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>{t('pi.provider.importDialog.title')}</DialogTitle>
          <DialogDescription>
            {step === 'channel' && t('pi.provider.importDialog.selectChannel')}
            {step === 'token' && t('pi.provider.importDialog.selectToken')}
            {step === 'configure' &&
              t('pi.provider.importDialog.configureProvider')}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto">
          {/* Step 1: Select Channel */}
          {step === 'channel' && (
            <div className="space-y-4 py-2">
              {enabledChannels.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-4">
                  {t('pi.provider.importDialog.noChannels')}
                </p>
              ) : (
                <>
                  <div className="space-y-2">
                    <Label>{t('pi.provider.importDialog.channel')}</Label>
                    <Select
                      value={selectedChannelId}
                      onValueChange={setSelectedChannelId}
                    >
                      <SelectTrigger>
                        <SelectValue
                          placeholder={t(
                            'pi.provider.importDialog.channelPlaceholder'
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

          {/* Step 2: Select Token */}
          {step === 'token' && selectedChannel && (
            <div className="space-y-3 py-2">
              {tokenFetchState?.isLoading ? (
                <div className="flex items-center justify-center py-6 gap-2">
                  <Loader2 className="h-5 w-5 animate-spin" />
                  <span className="text-sm text-muted-foreground">
                    {t('pi.provider.importDialog.loadingTokens')}
                  </span>
                </div>
              ) : tokenFetchState?.error ? (
                <div className="flex items-center gap-2 text-sm text-destructive py-2">
                  <AlertCircle className="h-4 w-4 shrink-0" />
                  <span>{tokenFetchState.error}</span>
                </div>
              ) : tokens.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-4">
                  {t('pi.provider.importDialog.noTokens')}
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
                        .filter(tk => tk.status === 1)
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

          {/* Step 3: Configure Provider */}
          {step === 'configure' && (
            <div className="space-y-4 py-2">
              <div className="space-y-2">
                <Label>{t('pi.provider.importDialog.providerId')} *</Label>
                <Input
                  value={providerId}
                  onChange={e => setProviderId(e.target.value)}
                  placeholder="my-provider"
                />
                {providerIdConflict && (
                  <p className="text-xs text-destructive">
                    {t('pi.provider.importDialog.providerIdConflict')}
                  </p>
                )}
              </div>

              <div className="rounded-md border px-3 py-2 bg-muted/40 space-y-1 text-sm">
                <div className="flex gap-2">
                  <span className="text-muted-foreground w-20 shrink-0">
                    {t('pi.provider.baseUrl')}
                  </span>
                  <span className="font-mono text-xs break-all">
                    {resolvedBaseUrl}
                  </span>
                </div>
                <div className="flex gap-2">
                  <span className="text-muted-foreground w-20 shrink-0">
                    {t('pi.provider.apiKey')}
                  </span>
                  <span className="font-mono text-xs">
                    {'•'.repeat(Math.min(resolvedApiKey.length, 16))}
                  </span>
                </div>
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label>{t('pi.provider.importDialog.models')}</Label>
                  {models.length > 0 && (
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="select-all-models"
                        checked={
                          selectedModelIds.size === models.length &&
                          models.length > 0
                        }
                        onCheckedChange={checked => toggleAllModels(!!checked)}
                      />
                      <Label
                        htmlFor="select-all-models"
                        className="text-xs cursor-pointer"
                      >
                        {t('pi.provider.importDialog.selectAll')}
                      </Label>
                    </div>
                  )}
                </div>
                {isFetchingModels ? (
                  <div className="flex items-center justify-center py-4 gap-2">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span className="text-sm text-muted-foreground">
                      {t('models.fetchingModels')}
                    </span>
                  </div>
                ) : modelError ? (
                  <div className="flex items-center gap-2 text-sm text-destructive">
                    <AlertCircle className="h-4 w-4 shrink-0" />
                    <span>{modelError}</span>
                  </div>
                ) : models.length === 0 ? (
                  <p className="text-xs text-muted-foreground py-2">
                    {t('pi.provider.importDialog.noModelsHint')}
                  </p>
                ) : (
                  <div className="border rounded-md max-h-48 overflow-y-auto">
                    {models.map(m => (
                      <div
                        key={m.id}
                        className="flex items-center gap-2 px-3 py-1.5 border-b last:border-b-0 hover:bg-muted/50"
                      >
                        <Checkbox
                          id={`model-${m.id}`}
                          checked={selectedModelIds.has(m.id)}
                          onCheckedChange={() => toggleModel(m.id)}
                        />
                        <Label
                          htmlFor={`model-${m.id}`}
                          className="flex-1 cursor-pointer text-xs font-mono"
                        >
                          {m.id}
                        </Label>
                      </div>
                    ))}
                  </div>
                )}
                <p className="text-xs text-muted-foreground">
                  {t('pi.provider.importDialog.modelsHint', {
                    count: selectedModelIds.size,
                  })}
                </p>
              </div>
            </div>
          )}
        </div>

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

          {step === 'configure' && (
            <Button onClick={handleImport} disabled={!canImport}>
              {t('pi.provider.importDialog.import')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
