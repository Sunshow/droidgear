import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, Search } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useChannelStore } from '@/store/channel-store'
import {
  commands,
  type CustomModel,
  type ModelInfo,
  type Provider,
} from '@/lib/bindings'
import {
  inferProviderFromPlatformAndModel,
  getBaseUrlForSub2Api,
} from '@/lib/sub2api-platform'
import {
  inferProviderForNewApi,
  getBaseUrlForNewApi,
} from '@/lib/newapi-platform'
import {
  containsRegexSpecialChars,
  getDefaultMaxOutputTokens,
} from '@/lib/utils'

export interface ChannelProviderContext {
  channelName: string
  baseUrl: string
  apiKey: string
  platform: string | null
}

interface ChannelModelPickerProps {
  mode: 'single' | 'multiple'
  existingModels?: CustomModel[]
  onSelect: (models: CustomModel[]) => void
  onSelectWithContext?: (
    models: CustomModel[],
    context: ChannelProviderContext
  ) => void
  showBatchConfig?: boolean
}

export function ChannelModelPicker({
  mode,
  existingModels = [],
  onSelect,
  onSelectWithContext,
  showBatchConfig = false,
}: ChannelModelPickerProps) {
  const { t } = useTranslation()

  const channels = useChannelStore(state => state.channels)
  const keys = useChannelStore(state => state.keys)
  const fetchKeys = useChannelStore(state => state.fetchKeys)
  const loadChannels = useChannelStore(state => state.loadChannels)

  const [selectedChannelId, setSelectedChannelId] = useState<string>('')
  const [selectedKeyId, setSelectedKeyId] = useState<string>('')
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([])
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [modelError, setModelError] = useState<string | null>(null)
  const [filterText, setFilterText] = useState('')

  // Selection state
  const [selectedModelIds, setSelectedModelIds] = useState<Set<string>>(
    new Set()
  )
  const [singleSelectedId, setSingleSelectedId] = useState<string>('')

  // Batch config state
  const [prefix, setPrefix] = useState('')
  const [suffix, setSuffix] = useState('')
  const [batchMaxTokens, setBatchMaxTokens] = useState('')
  const [batchSupportsImages, setBatchSupportsImages] = useState(false)

  const enabledChannels = channels.filter(ch => ch.enabled)
  const selectedChannel = enabledChannels.find(
    ch => ch.id === selectedChannelId
  )
  const channelKeys = selectedChannelId ? (keys[selectedChannelId] ?? []) : []
  const selectedKey = channelKeys.find(k => String(k.id) === selectedKeyId)

  useEffect(() => {
    if (channels.length === 0) {
      loadChannels()
    }
  }, [channels.length, loadChannels])

  const handleChannelChange = async (channelId: string) => {
    setSelectedChannelId(channelId)
    setSelectedKeyId('')
    setAvailableModels([])
    setSelectedModelIds(new Set())
    setSingleSelectedId('')
    setModelError(null)

    const channel = enabledChannels.find(ch => ch.id === channelId)
    if (channel && !keys[channelId]) {
      await fetchKeys(channelId, channel.type, channel.baseUrl)
    }
  }

  const handleKeyChange = async (keyId: string) => {
    setSelectedKeyId(keyId)
    setAvailableModels([])
    setSelectedModelIds(new Set())
    setSingleSelectedId('')
    setModelError(null)

    const key = channelKeys.find(k => String(k.id) === keyId)
    if (!key || !selectedChannel) return

    setIsFetchingModels(true)
    const result = await commands.fetchModelsByApiKey(
      selectedChannel.baseUrl,
      key.key,
      key.platform
    )
    setIsFetchingModels(false)

    if (result.status === 'ok') {
      setAvailableModels(result.data)
      if (result.data.length === 0) {
        setModelError(t('models.noModelsAvailable'))
      }
    } else {
      setModelError(result.error)
    }
  }

  const inferProvider = (modelId: string): Provider => {
    if (!selectedChannel || !selectedKey) return 'generic-chat-completion-api'
    // CLI Proxy API and New API use the same logic
    if (
      selectedChannel.type === 'new-api' ||
      selectedChannel.type === 'cli-proxy-api'
    ) {
      return inferProviderForNewApi(modelId)
    }
    return inferProviderFromPlatformAndModel(selectedKey.platform, modelId)
  }

  const getBaseUrl = (provider: Provider): string => {
    if (!selectedChannel || !selectedKey) return ''
    // CLI Proxy API and New API use the same baseUrl logic
    if (
      selectedChannel.type === 'new-api' ||
      selectedChannel.type === 'cli-proxy-api'
    ) {
      return getBaseUrlForNewApi(provider, selectedChannel.baseUrl)
    }
    return getBaseUrlForSub2Api(
      provider,
      selectedChannel.baseUrl,
      selectedKey.platform
    )
  }

  const isModelExisting = (modelId: string): boolean => {
    return existingModels.some(
      m => m.model === modelId && m.apiKey === selectedKey?.key
    )
  }

  const buildCustomModel = (modelId: string): CustomModel => {
    const provider = inferProvider(modelId)
    const baseUrl = getBaseUrl(provider)

    let displayName = modelId
    if (prefix || suffix) {
      displayName = `${prefix}${modelId}${suffix}`
    }

    let maxOutputTokens: number | undefined
    if (batchMaxTokens) {
      maxOutputTokens = parseInt(batchMaxTokens)
    } else {
      maxOutputTokens = getDefaultMaxOutputTokens(modelId)
    }

    return {
      model: modelId,
      baseUrl,
      apiKey: selectedKey?.key ?? '',
      provider,
      displayName,
      maxOutputTokens,
      supportsImages: batchSupportsImages || undefined,
    }
  }

  const buildProviderContext = (): ChannelProviderContext | null => {
    if (!selectedChannel || !selectedKey) return null
    return {
      channelName: selectedChannel.name,
      baseUrl: selectedChannel.baseUrl,
      apiKey: selectedKey.key,
      platform: selectedKey.platform,
    }
  }

  const handleSingleSelect = (modelId: string) => {
    setSingleSelectedId(modelId)
    const model = buildCustomModel(modelId)
    if (onSelectWithContext) {
      const context = buildProviderContext()
      if (context) {
        onSelectWithContext([model], context)
        return
      }
    }
    onSelect([model])
  }

  const handleToggleModel = (modelId: string) => {
    setSelectedModelIds(prev => {
      const next = new Set(prev)
      if (next.has(modelId)) {
        next.delete(modelId)
      } else {
        next.add(modelId)
      }
      return next
    })
  }

  const selectableModels = availableModels.filter(m => !isModelExisting(m.id))

  const handleSelectAll = () => {
    if (selectedModelIds.size === selectableModels.length) {
      setSelectedModelIds(new Set())
    } else {
      setSelectedModelIds(new Set(selectableModels.map(m => m.id)))
    }
  }

  const handleConfirmSelection = () => {
    const models = Array.from(selectedModelIds)
      .filter(id => !isModelExisting(id))
      .map(id => buildCustomModel(id))
    if (onSelectWithContext) {
      const context = buildProviderContext()
      if (context) {
        onSelectWithContext(models, context)
        return
      }
    }
    onSelect(models)
  }

  const filteredModels = filterText
    ? availableModels.filter(m => {
        const searchLower = filterText.toLowerCase()
        return (
          m.id.toLowerCase().includes(searchLower) ||
          m.name?.toLowerCase().includes(searchLower)
        )
      })
    : availableModels

  const isBatchConfigValid =
    !containsRegexSpecialChars(prefix) && !containsRegexSpecialChars(suffix)

  const canConfirm =
    mode === 'multiple' && selectedModelIds.size > 0 && isBatchConfigValid

  return (
    <div className="space-y-4">
      {/* Channel Select */}
      <div className="space-y-2">
        <Label>{t('channels.selectChannel')}</Label>
        <Select value={selectedChannelId} onValueChange={handleChannelChange}>
          <SelectTrigger>
            <SelectValue placeholder={t('channels.selectChannelPlaceholder')} />
          </SelectTrigger>
          <SelectContent>
            {enabledChannels.map(ch => (
              <SelectItem key={ch.id} value={ch.id}>
                {ch.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {enabledChannels.length === 0 && (
          <p className="text-sm text-muted-foreground">
            {t('channels.noEnabledChannels')}
          </p>
        )}
      </div>

      {/* Key Select */}
      {selectedChannelId && (
        <div className="space-y-2">
          <Label>{t('channels.selectKey')}</Label>
          <Select value={selectedKeyId} onValueChange={handleKeyChange}>
            <SelectTrigger>
              <SelectValue placeholder={t('channels.selectKeyPlaceholder')} />
            </SelectTrigger>
            <SelectContent>
              {channelKeys.map(k => (
                <SelectItem key={k.id} value={String(k.id)}>
                  {k.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {channelKeys.length === 0 && (
            <p className="text-sm text-muted-foreground">
              {t('channels.noKeysAvailable')}
            </p>
          )}
        </div>
      )}

      {/* Batch Config */}
      {showBatchConfig && mode === 'multiple' && selectedKeyId && (
        <div className="space-y-4 pt-2 border-t">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="picker-prefix">{t('models.prefix')}</Label>
              <Input
                id="picker-prefix"
                value={prefix}
                onChange={e => setPrefix(e.target.value)}
                placeholder={t('models.prefixPlaceholder')}
              />
              {containsRegexSpecialChars(prefix) && (
                <p className="text-sm text-destructive">
                  {t('validation.bracketsNotAllowed')}
                </p>
              )}
            </div>
            <div className="space-y-2">
              <Label htmlFor="picker-suffix">{t('models.suffix')}</Label>
              <Input
                id="picker-suffix"
                value={suffix}
                onChange={e => setSuffix(e.target.value)}
                placeholder={t('models.suffixPlaceholder')}
              />
              {containsRegexSpecialChars(suffix) && (
                <p className="text-sm text-destructive">
                  {t('validation.bracketsNotAllowed')}
                </p>
              )}
            </div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="picker-max-tokens">
                {t('models.batchMaxTokens')}
              </Label>
              <Input
                id="picker-max-tokens"
                type="number"
                value={batchMaxTokens}
                onChange={e => setBatchMaxTokens(e.target.value)}
                placeholder={t('models.maxTokensPlaceholder')}
                step={8192}
              />
            </div>
            <div className="flex items-end gap-2 pb-2">
              <Checkbox
                id="picker-supports-images"
                checked={batchSupportsImages}
                onCheckedChange={checked =>
                  setBatchSupportsImages(checked === true)
                }
              />
              <Label htmlFor="picker-supports-images">
                {t('models.batchSupportsImages')}
              </Label>
            </div>
          </div>
        </div>
      )}

      {/* Model List */}
      {selectedKeyId && (
        <div className="space-y-2">
          {isFetchingModels ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin mr-2" />
              <span>{t('models.fetchingModels')}</span>
            </div>
          ) : modelError ? (
            <div className="py-4 text-center text-destructive">
              <p>{modelError}</p>
            </div>
          ) : availableModels.length === 0 ? (
            <div className="py-4 text-center text-muted-foreground">
              <p>{t('models.noModelsAvailable')}</p>
            </div>
          ) : (
            <>
              {/* Filter */}
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  value={filterText}
                  onChange={e => setFilterText(e.target.value)}
                  placeholder={t('models.filterModels')}
                  className="pl-9"
                />
              </div>

              {/* Select header for multiple mode */}
              {mode === 'multiple' && (
                <div className="flex items-center justify-between">
                  <Label>
                    {t('models.selectModelsToAdd', {
                      count: selectedModelIds.size,
                    })}
                  </Label>
                  <Button variant="ghost" size="sm" onClick={handleSelectAll}>
                    {selectedModelIds.size === selectableModels.length
                      ? t('common.deselectAll')
                      : t('common.selectAll')}
                  </Button>
                </div>
              )}

              {/* Model list */}
              <div className="border rounded-md p-2 overflow-auto max-h-[240px]">
                {mode === 'single' ? (
                  <RadioGroup
                    value={singleSelectedId}
                    onValueChange={handleSingleSelect}
                  >
                    {filteredModels.map(m => {
                      const isExisting = isModelExisting(m.id)
                      return (
                        <div
                          key={m.id}
                          className="flex items-center gap-2 p-2 rounded hover:bg-accent/50"
                        >
                          <RadioGroupItem
                            value={m.id}
                            id={`single-model-${m.id}`}
                            disabled={isExisting}
                          />
                          <label
                            htmlFor={`single-model-${m.id}`}
                            className={`text-sm cursor-pointer flex-1 ${isExisting ? 'text-muted-foreground' : ''}`}
                          >
                            {m.name || m.id}
                            {isExisting && (
                              <span className="ml-2 text-xs">
                                {t('models.alreadyAddedForKey')}
                              </span>
                            )}
                          </label>
                        </div>
                      )
                    })}
                  </RadioGroup>
                ) : (
                  <div className="space-y-1">
                    {filteredModels.map(m => {
                      const isExisting = isModelExisting(m.id)
                      const isSelected = selectedModelIds.has(m.id)
                      return (
                        <div
                          key={m.id}
                          className="flex items-center gap-2 p-2 rounded hover:bg-accent/50"
                        >
                          <Checkbox
                            id={`multi-model-${m.id}`}
                            checked={isSelected}
                            onCheckedChange={() => handleToggleModel(m.id)}
                            disabled={isExisting}
                          />
                          <label
                            htmlFor={`multi-model-${m.id}`}
                            className={`text-sm cursor-pointer flex-1 ${isExisting ? 'text-muted-foreground' : ''}`}
                          >
                            {m.name || m.id}
                            {isExisting && (
                              <span className="ml-2 text-xs">
                                {t('models.alreadyAddedForKey')}
                              </span>
                            )}
                          </label>
                        </div>
                      )
                    })}
                  </div>
                )}
              </div>

              {/* Confirm button for multiple mode */}
              {mode === 'multiple' && (
                <div className="flex justify-end pt-2">
                  <Button
                    onClick={handleConfirmSelection}
                    disabled={!canConfirm}
                  >
                    {selectedModelIds.size === 1
                      ? t('models.addCount', { count: selectedModelIds.size })
                      : t('models.addCountPlural', {
                          count: selectedModelIds.size,
                        })}
                  </Button>
                </div>
              )}
            </>
          )}
        </div>
      )}
    </div>
  )
}
