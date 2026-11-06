import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertCircle, ChevronLeft, ChevronRight, Loader2 } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { isApiKeyAuthChannel } from '@/lib/channel-utils'
import {
  commands,
  type Channel,
  type ChannelToken,
  type ModelInfo,
  type OmpCustomProvider,
  type OmpCustomProviderEntry,
} from '@/lib/bindings'
import { useChannelStore } from '@/store/channel-store'

interface OmpImportFromChannelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  existingProviders: readonly OmpCustomProviderEntry[]
  /**
   * When set, the dialog updates this provider (ID locked, models merged)
   * instead of creating a new one.
   */
  targetProviderId?: string | null
  onImported: (
    providerId: string,
    provider: OmpCustomProvider
  ) => void | Promise<void>
}

type Step = 'channel' | 'token' | 'configure'

const channelTypeI18nKeys: Record<string, string> = {
  'new-api': 'channels.typeNewApi',
  'sub-2-api': 'channels.typeSub2Api',
  'cli-proxy-api': 'channels.typeCliProxyApi',
  ollama: 'channels.typeOllama',
  general: 'channels.typeGeneral',
  'deep-seek': 'channels.typeDeepSeek',
}

export function OmpImportFromChannelDialog({
  open,
  onOpenChange,
  existingProviders,
  targetProviderId = null,
  onImported,
}: OmpImportFromChannelDialogProps) {
  const { t } = useTranslation()
  const channels = useChannelStore(state => state.channels)
  const loadChannels = useChannelStore(state => state.loadChannels)
  const keysMap = useChannelStore(state => state.keys)
  const fetchKeys = useChannelStore(state => state.fetchKeys)
  const keysFetchState = useChannelStore(state => state.keysFetchState)

  const isUpdate = !!targetProviderId
  const targetProvider = isUpdate
    ? (existingProviders.find(entry => entry.providerId === targetProviderId)
        ?.config ?? null)
    : null
  const existingProviderIds = existingProviders.map(entry => entry.providerId)

  const [step, setStep] = useState<Step>('channel')
  const [selectedChannelId, setSelectedChannelId] = useState('')
  const [resolvedApiKey, setResolvedApiKey] = useState('')
  const [resolvedBaseUrl, setResolvedBaseUrl] = useState('')
  const [resolvedPlatform, setResolvedPlatform] = useState<string | null>(null)
  const [resolvedTokenName, setResolvedTokenName] = useState('')
  const [resolvedApi, setResolvedApi] = useState('openai-completions')
  const [providerId, setProviderId] = useState('')
  const [models, setModels] = useState<ModelInfo[]>([])
  const [selectedModelIds, setSelectedModelIds] = useState<Set<string>>(
    new Set()
  )
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [modelError, setModelError] = useState<string | null>(null)
  const [isResolvingKey, setIsResolvingKey] = useState(false)
  const [resolveError, setResolveError] = useState<string | null>(null)

  const selectedChannel: Channel | undefined = channels.find(
    channel => channel.id === selectedChannelId
  )
  const tokens: ChannelToken[] = keysMap[selectedChannelId] ?? []
  const tokenFetchState = keysFetchState[selectedChannelId]

  // Reset state when the dialog opens
  useEffect(() => {
    if (!open) return
    setStep('channel')
    setSelectedChannelId('')
    setResolvedApiKey('')
    setResolvedBaseUrl('')
    setResolvedPlatform(null)
    setResolvedTokenName('')
    setResolvedApi('openai-completions')
    setProviderId(targetProviderId ?? '')
    setModels([])
    setSelectedModelIds(new Set())
    setModelError(null)
    setResolveError(null)
    void loadChannels()
  }, [open, targetProviderId, loadChannels])

  // Fetch tokens for credential-based channels
  useEffect(() => {
    if (
      selectedChannelId &&
      selectedChannel &&
      !isApiKeyAuthChannel(selectedChannel.type) &&
      keysMap[selectedChannelId] === undefined
    ) {
      void fetchKeys(
        selectedChannelId,
        selectedChannel.type,
        selectedChannel.baseUrl
      )
    }
  }, [selectedChannelId, selectedChannel, keysMap, fetchKeys])

  const fetchModelsForKey = async (
    baseUrl: string,
    apiKey: string,
    platform: string | null | undefined,
    tokenName: string
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
        setSelectedModelIds(new Set(result.data.map(model => model.id)))
        // The service owns the inference (base URL, protocol, provider id);
        // the dialog asks it once the model list is known and reuses the
        // answer on import, so GUI/CLI/TUI cannot drift.
        const prepared = await commands.prepareOmpChannelImport({
          channelName: selectedChannel?.name ?? '',
          channelType: selectedChannel?.type ?? '',
          baseUrl: selectedChannel?.baseUrl ?? baseUrl,
          platform: platform ?? null,
          apiKey,
          tokenName,
          providerId: isUpdate ? (targetProviderId ?? '') : '',
          modelIds: result.data.map(model => model.id),
          channelModels: result.data.map(model => ({
            id: model.id,
            name: model.name ?? null,
          })),
          existingProviderIds,
          existing: targetProvider ?? undefined,
        })
        if (prepared.status === 'ok') {
          setResolvedBaseUrl(prepared.data.provider.baseUrl ?? '')
          setResolvedApi(prepared.data.provider.api ?? 'openai-completions')
          if (!isUpdate && !providerId.trim()) {
            setProviderId(prepared.data.providerId)
          }
        } else {
          setModelError(prepared.error)
        }
      } else {
        setModelError(result.error)
      }
    } catch (error) {
      setModelError(String(error))
    } finally {
      setIsFetchingModels(false)
    }
  }

  const handleChannelNext = async () => {
    if (!selectedChannel) return
    setResolveError(null)

    if (!isApiKeyAuthChannel(selectedChannel.type)) {
      setStep('token')
      return
    }

    setIsResolvingKey(true)
    try {
      const result = await commands.getChannelApiKey(selectedChannel.id)
      if (result.status !== 'ok' || !result.data) {
        setResolveError(
          t('omp.provider.importDialog.noApiKey', {
            channel: selectedChannel.name,
          })
        )
        return
      }
      setResolvedApiKey(result.data)
      setResolvedPlatform(null)
      setResolvedTokenName('')
      await fetchModelsForKey(selectedChannel.baseUrl, result.data, null, '')
      setStep('configure')
    } catch (error) {
      setResolveError(String(error))
    } finally {
      setIsResolvingKey(false)
    }
  }

  const handleTokenSelect = async (token: ChannelToken) => {
    if (!selectedChannel) return
    setResolvedApiKey(token.key)
    setResolvedPlatform(token.platform ?? null)
    setResolvedTokenName(token.name)
    await fetchModelsForKey(
      selectedChannel.baseUrl,
      token.key,
      token.platform,
      token.name
    )
    setStep('configure')
  }

  const toggleModel = (id: string) => {
    setSelectedModelIds(previous => {
      const next = new Set(previous)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const toggleAllModels = (checked: boolean) => {
    setSelectedModelIds(
      checked ? new Set(models.map(model => model.id)) : new Set()
    )
  }

  const handleImport = async () => {
    if (!selectedChannel) return
    const trimmedProviderId = providerId.trim()
    if (!trimmedProviderId) return

    const prepared = await commands.prepareOmpChannelImport({
      channelName: selectedChannel.name,
      channelType: selectedChannel.type,
      baseUrl: selectedChannel.baseUrl,
      platform: resolvedPlatform,
      apiKey: resolvedApiKey,
      tokenName: resolvedTokenName,
      providerId: trimmedProviderId,
      modelIds: models
        .filter(model => selectedModelIds.has(model.id))
        .map(model => model.id),
      channelModels: models
        .filter(model => selectedModelIds.has(model.id))
        .map(model => ({ id: model.id, name: model.name ?? null })),
      existingProviderIds,
      existing: targetProvider ?? undefined,
    })
    if (prepared.status !== 'ok') {
      setModelError(prepared.error)
      return
    }
    await onImported(prepared.data.providerId, prepared.data.provider)
    onOpenChange(false)
  }

  const handleBack = () => {
    if (step === 'configure') {
      setStep(
        selectedChannel && isApiKeyAuthChannel(selectedChannel.type)
          ? 'channel'
          : 'token'
      )
    } else if (step === 'token') {
      setStep('channel')
    }
  }

  const enabledChannels = channels.filter(channel => channel.enabled)
  const trimmedProviderId = providerId.trim()
  const providerIdConflict =
    !isUpdate &&
    !!trimmedProviderId &&
    existingProviderIds.includes(trimmedProviderId)
  const canImport = !!trimmedProviderId && !providerIdConflict
  const importedApi = resolvedApi

  const renderChannelStep = () => {
    if (enabledChannels.length > 0) {
      return (
        <>
          <div className="space-y-2">
            <Label>{t('omp.provider.importDialog.channel')}</Label>
            <Select
              value={selectedChannelId}
              onValueChange={setSelectedChannelId}
            >
              <SelectTrigger>
                <SelectValue
                  placeholder={t(
                    'omp.provider.importDialog.channelPlaceholder'
                  )}
                />
              </SelectTrigger>
              <SelectContent>
                {enabledChannels.map(channel => (
                  <SelectItem key={channel.id} value={channel.id}>
                    <span className="font-medium">{channel.name}</span>
                    <span className="ml-2 text-muted-foreground text-xs">
                      {t(channelTypeI18nKeys[channel.type] ?? channel.type)}
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedChannel && (
            <div className="text-sm text-muted-foreground rounded-md border px-3 py-2 bg-muted/40 break-all">
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
      )
    }
    return (
      <p className="text-sm text-muted-foreground text-center py-4">
        {t('omp.provider.importDialog.noChannels')}
      </p>
    )
  }

  const renderTokenStep = () => {
    if (tokenFetchState?.isLoading) {
      return (
        <div className="flex items-center justify-center py-6 gap-2">
          <Loader2 className="h-5 w-5 animate-spin" />
          <span className="text-sm text-muted-foreground">
            {t('omp.provider.importDialog.loadingTokens')}
          </span>
        </div>
      )
    }
    if (tokenFetchState?.error) {
      return (
        <div className="flex items-center gap-2 text-sm text-destructive py-2">
          <AlertCircle className="h-4 w-4 shrink-0" />
          <span>{tokenFetchState.error}</span>
        </div>
      )
    }
    const activeTokens = tokens.filter(token => token.status === 1)
    if (activeTokens.length === 0) {
      return (
        <p className="text-sm text-muted-foreground text-center py-4">
          {t('omp.provider.importDialog.noTokens')}
        </p>
      )
    }
    return (
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
            {activeTokens.map(token => (
              <TableRow key={token.id}>
                <TableCell className="font-medium">{token.name}</TableCell>
                <TableCell>
                  {token.platform ? (
                    <Badge variant="outline">{token.platform}</Badge>
                  ) : (
                    <span className="text-muted-foreground">—</span>
                  )}
                </TableCell>
                <TableCell>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => void handleTokenSelect(token)}
                  >
                    {t('common.select')}
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    )
  }

  const renderConfigureStep = () => {
    const allSelected =
      models.length > 0 && selectedModelIds.size === models.length
    return (
      <>
        <div className="space-y-2">
          <Label>{t('omp.provider.importDialog.providerId')} *</Label>
          <Input
            value={providerId}
            onChange={event => setProviderId(event.target.value)}
            disabled={isUpdate}
            placeholder="my-provider"
          />
          {isUpdate && (
            <p className="text-xs text-muted-foreground">
              {t('omp.provider.importDialog.providerIdLocked')}
            </p>
          )}
          {providerIdConflict && (
            <p className="text-xs text-destructive">
              {t('omp.provider.importDialog.providerIdConflict')}
            </p>
          )}
        </div>

        <div className="rounded-md border px-3 py-2 bg-muted/40 space-y-1 text-sm">
          <div className="flex gap-2">
            <span className="text-muted-foreground w-20 shrink-0">
              {t('omp.provider.baseUrl')}
            </span>
            <span className="font-mono text-xs break-all">
              {resolvedBaseUrl}
            </span>
          </div>
          <div className="flex gap-2">
            <span className="text-muted-foreground w-20 shrink-0">
              {t('omp.provider.apiKey')}
            </span>
            <span className="font-mono text-xs">
              {'•'.repeat(Math.min(resolvedApiKey.length, 16))}
            </span>
          </div>
          <div className="flex gap-2">
            <span className="text-muted-foreground w-20 shrink-0">
              {t('omp.provider.api')}
            </span>
            <span className="font-mono text-xs">{importedApi}</span>
          </div>
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label>{t('omp.provider.importDialog.models')}</Label>
            {models.length > 0 && (
              <div className="flex items-center gap-2">
                <Checkbox
                  id="omp-select-all-models"
                  checked={allSelected}
                  onCheckedChange={checked => toggleAllModels(!!checked)}
                />
                <Label
                  htmlFor="omp-select-all-models"
                  className="text-xs cursor-pointer"
                >
                  {t('omp.provider.importDialog.selectAll')}
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
              {t('omp.provider.importDialog.noModelsHint')}
            </p>
          ) : (
            <div className="border rounded-md max-h-48 overflow-y-auto">
              {models.map(model => (
                <div
                  key={model.id}
                  className="flex items-center gap-2 px-3 py-1.5 border-b last:border-b-0 hover:bg-muted/50"
                >
                  <Checkbox
                    id={`omp-model-${model.id}`}
                    checked={selectedModelIds.has(model.id)}
                    onCheckedChange={() => toggleModel(model.id)}
                  />
                  <Label
                    htmlFor={`omp-model-${model.id}`}
                    className="flex-1 cursor-pointer text-xs font-mono"
                  >
                    {model.id}
                  </Label>
                </div>
              ))}
            </div>
          )}
          <p className="text-xs text-muted-foreground">
            {t('omp.provider.importDialog.modelsHint', {
              count: selectedModelIds.size,
            })}
          </p>
        </div>
      </>
    )
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px] max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>
            {isUpdate
              ? t('omp.provider.importDialog.updateTitle')
              : t('omp.provider.importDialog.title')}
          </DialogTitle>
          <DialogDescription>
            {step === 'channel' && t('omp.provider.importDialog.selectChannel')}
            {step === 'token' && t('omp.provider.importDialog.selectToken')}
            {step === 'configure' &&
              t(
                isUpdate
                  ? 'omp.provider.importDialog.configureExisting'
                  : 'omp.provider.importDialog.configureProvider'
              )}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto">
          {step === 'channel' && (
            <div className="space-y-4 py-2">{renderChannelStep()}</div>
          )}
          {step === 'token' && selectedChannel && (
            <div className="space-y-3 py-2">{renderTokenStep()}</div>
          )}
          {step === 'configure' && (
            <div className="space-y-4 py-2">{renderConfigureStep()}</div>
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
              onClick={() => void handleChannelNext()}
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
            <Button onClick={() => void handleImport()} disabled={!canImport}>
              {t(
                isUpdate
                  ? 'omp.provider.importDialog.update'
                  : 'omp.provider.importDialog.import'
              )}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
