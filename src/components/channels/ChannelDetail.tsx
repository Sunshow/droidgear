import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Pencil, Trash2, AlertCircle, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  ResizableDialog,
  ResizableDialogContent,
  ResizableDialogHeader,
  ResizableDialogBody,
  ResizableDialogTitle,
  ResizableDialogFooter,
} from '@/components/ui/resizable-dialog'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { KeyList } from './KeyList'
import { useChannelStore } from '@/store/channel-store'
import { useModelStore } from '@/store/model-store'
import {
  commands,
  type Channel,
  type ChannelToken,
  type ChannelType,
  type ModelInfo,
  type CustomModel,
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

const channelTypeI18nKeys: Record<ChannelType, string> = {
  'new-api': 'channels.typeNewApi',
  'sub-2-api': 'channels.typeSub2Api',
}

interface ChannelDetailProps {
  channel: Channel
  onEdit: () => void
}

export function ChannelDetail({ channel, onEdit }: ChannelDetailProps) {
  const { t } = useTranslation()
  const deleteChannel = useChannelStore(state => state.deleteChannel)
  const saveChannels = useChannelStore(state => state.saveChannels)
  const error = useChannelStore(state => state.error)
  const setError = useChannelStore(state => state.setError)

  const addModel = useModelStore(state => state.addModel)
  const saveModels = useModelStore(state => state.saveModels)
  const loadModels = useModelStore(state => state.loadModels)
  const existingModels = useModelStore(state => state.models)

  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [modelDialogOpen, setModelDialogOpen] = useState(false)
  const [selectedKey, setSelectedKey] = useState<ChannelToken | null>(null)
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([])
  // Map of modelId -> { alias, provider }
  const [selectedModels, setSelectedModels] = useState<
    Map<string, { alias: string; provider: Provider }>
  >(new Map())
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [modelError, setModelError] = useState<string | null>(null)
  // Prefix and suffix for batch alias generation
  const [prefix, setPrefix] = useState('')
  const [suffix, setSuffix] = useState('')

  // Load models on mount
  useEffect(() => {
    loadModels()
  }, [loadModels])

  const handleDelete = async () => {
    await deleteChannel(channel.id)
    await saveChannels()
    setDeleteDialogOpen(false)
  }

  const handleSelectKey = async (apiKey: ChannelToken) => {
    setSelectedKey(apiKey)
    setModelDialogOpen(true)
    setIsFetchingModels(true)
    setModelError(null)
    setSelectedModels(new Map())
    setPrefix('')
    setSuffix('')

    const result = await commands.fetchModelsByApiKey(
      channel.baseUrl,
      apiKey.key,
      apiKey.platform
    )
    setIsFetchingModels(false)

    if (result.status === 'ok') {
      setAvailableModels(result.data)
    } else {
      setModelError(result.error)
    }
  }

  const handleToggleModel = (modelId: string) => {
    setSelectedModels(prev => {
      const next = new Map(prev)
      if (next.has(modelId)) {
        next.delete(modelId)
      } else {
        const defaultProvider =
          channel.type === 'new-api'
            ? inferProviderForNewApi(modelId)
            : inferProviderFromPlatformAndModel(selectedKey?.platform, modelId)
        next.set(modelId, { alias: '', provider: defaultProvider })
      }
      return next
    })
  }

  const handleAliasChange = (modelId: string, alias: string) => {
    setSelectedModels(prev => {
      const next = new Map(prev)
      const current = next.get(modelId)
      if (current) {
        next.set(modelId, { ...current, alias })
      }
      return next
    })
  }

  const handleProviderChange = (modelId: string, provider: Provider) => {
    setSelectedModels(prev => {
      const next = new Map(prev)
      const current = next.get(modelId)
      if (current) {
        next.set(modelId, { ...current, provider })
      }
      return next
    })
  }

  // Check if model+key combination already exists
  const isModelKeyExisting = (modelId: string, apiKeyValue: string) => {
    return existingModels.some(
      m => m.model === modelId && m.apiKey === apiKeyValue
    )
  }

  const handleAddModels = async () => {
    if (!selectedKey || selectedModels.size === 0) return

    for (const [modelId, config] of selectedModels) {
      // Skip if this model+key combination already exists
      if (isModelKeyExisting(modelId, selectedKey.key)) continue

      const baseUrl =
        channel.type === 'new-api'
          ? getBaseUrlForNewApi(config.provider, channel.baseUrl)
          : getBaseUrlForSub2Api(
              config.provider,
              channel.baseUrl,
              selectedKey?.platform
            )

      // Determine display name: custom alias > prefix+model+suffix > model
      let displayName = modelId
      if (config.alias) {
        displayName = config.alias
      } else if (prefix || suffix) {
        displayName = `${prefix}${modelId}${suffix}`
      }

      const newModel: CustomModel = {
        model: modelId,
        baseUrl,
        apiKey: selectedKey.key,
        provider: config.provider,
        displayName,
        maxOutputTokens: getDefaultMaxOutputTokens(modelId),
      }
      addModel(newModel)
    }

    await saveModels()
    setModelDialogOpen(false)
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-semibold">{channel.name}</h1>
            {!channel.enabled && (
              <Badge variant="secondary">{t('common.disabled')}</Badge>
            )}
          </div>
          <p className="text-sm text-muted-foreground mt-1">
            {t(channelTypeI18nKeys[channel.type])} - {channel.baseUrl}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={onEdit}>
            <Pencil className="h-4 w-4 mr-2" />
            {t('common.edit')}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setDeleteDialogOpen(true)}
          >
            <Trash2 className="h-4 w-4 mr-2" />
            {t('common.delete')}
          </Button>
        </div>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md flex items-center gap-2">
          <AlertCircle className="h-4 w-4 text-destructive" />
          <span className="text-sm text-destructive">{error}</span>
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto"
            onClick={() => setError(null)}
          >
            {t('common.dismiss')}
          </Button>
        </div>
      )}

      {/* Key List */}
      <div className="flex-1 overflow-auto p-4">
        <KeyList
          channelId={channel.id}
          channelType={channel.type}
          baseUrl={channel.baseUrl}
          onSelectKey={handleSelectKey}
        />
      </div>

      {/* Delete Confirmation */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('channels.deleteChannel')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('channels.deleteConfirm', { name: channel.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Model Selection Dialog */}
      <ResizableDialog open={modelDialogOpen} onOpenChange={setModelDialogOpen}>
        <ResizableDialogContent
          defaultWidth={700}
          defaultHeight={550}
          minWidth={500}
          minHeight={400}
        >
          <ResizableDialogHeader>
            <ResizableDialogTitle>{t('models.addModels')}</ResizableDialogTitle>
          </ResizableDialogHeader>

          <ResizableDialogBody>
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
              <div className="space-y-4">
                {/* Prefix and Suffix inputs */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="prefix">{t('models.prefix')}</Label>
                    <Input
                      id="prefix"
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
                    <Label htmlFor="suffix">{t('models.suffix')}</Label>
                    <Input
                      id="suffix"
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

                <div className="flex items-center justify-between">
                  <Label>
                    {t('models.selectModelsToAdd', {
                      count: selectedModels.size,
                    })}
                  </Label>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      const selectableModels = availableModels.filter(
                        m => !isModelKeyExisting(m.id, selectedKey?.key ?? '')
                      )
                      if (selectedModels.size === selectableModels.length) {
                        setSelectedModels(new Map())
                      } else {
                        const newMap = new Map<
                          string,
                          { alias: string; provider: Provider }
                        >()
                        selectableModels.forEach(m => {
                          const provider =
                            channel.type === 'new-api'
                              ? inferProviderForNewApi(m.id)
                              : inferProviderFromPlatformAndModel(
                                  selectedKey?.platform,
                                  m.id
                                )
                          newMap.set(m.id, { alias: '', provider })
                        })
                        setSelectedModels(newMap)
                      }
                    }}
                  >
                    {selectedModels.size ===
                    availableModels.filter(
                      m => !isModelKeyExisting(m.id, selectedKey?.key ?? '')
                    ).length
                      ? t('common.deselectAll')
                      : t('common.selectAll')}
                  </Button>
                </div>
                <div className="flex-1 border rounded-md p-2 overflow-auto">
                  <div className="space-y-2">
                    {availableModels.map(model => {
                      const isExisting = isModelKeyExisting(
                        model.id,
                        selectedKey?.key ?? ''
                      )
                      const isSelected = selectedModels.has(model.id)
                      const modelConfig = selectedModels.get(model.id)
                      return (
                        <div
                          key={model.id}
                          className="flex items-center gap-2 p-2 rounded hover:bg-accent/50"
                        >
                          <Checkbox
                            id={model.id}
                            checked={isSelected}
                            onCheckedChange={() => handleToggleModel(model.id)}
                            disabled={isExisting}
                          />
                          <label
                            htmlFor={model.id}
                            className="text-sm cursor-pointer min-w-[120px] shrink-0"
                          >
                            {model.name || model.id}
                            {isExisting && (
                              <span className="ml-2 text-xs text-muted-foreground">
                                {t('models.alreadyAddedForKey')}
                              </span>
                            )}
                          </label>
                          {isSelected && (
                            <>
                              <Select
                                value={modelConfig?.provider}
                                onValueChange={(value: Provider) =>
                                  handleProviderChange(model.id, value)
                                }
                              >
                                <SelectTrigger className="h-7 w-[140px] text-sm shrink-0">
                                  <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                  <SelectItem value="anthropic">
                                    {t('models.providerAnthropic')}
                                  </SelectItem>
                                  <SelectItem value="openai">
                                    {t('models.providerOpenAI')}
                                  </SelectItem>
                                  <SelectItem value="generic-chat-completion-api">
                                    {t('models.providerGeneric')}
                                  </SelectItem>
                                </SelectContent>
                              </Select>
                              <div className="flex-1 space-y-1">
                                <Input
                                  className="h-7 text-sm"
                                  value={modelConfig?.alias ?? ''}
                                  onChange={e =>
                                    handleAliasChange(model.id, e.target.value)
                                  }
                                  placeholder={t('models.aliasPlaceholder')}
                                />
                                {containsRegexSpecialChars(
                                  modelConfig?.alias ?? ''
                                ) && (
                                  <p className="text-xs text-destructive">
                                    {t('validation.bracketsNotAllowed')}
                                  </p>
                                )}
                              </div>
                            </>
                          )}
                        </div>
                      )
                    })}
                  </div>
                </div>
              </div>
            )}
          </ResizableDialogBody>

          <ResizableDialogFooter>
            <Button variant="outline" onClick={() => setModelDialogOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleAddModels}
              disabled={selectedModels.size === 0 || isFetchingModels}
            >
              {selectedModels.size === 1
                ? t('models.addCount', { count: selectedModels.size })
                : t('models.addCountPlural', { count: selectedModels.size })}
            </Button>
          </ResizableDialogFooter>
        </ResizableDialogContent>
      </ResizableDialog>
    </div>
  )
}
