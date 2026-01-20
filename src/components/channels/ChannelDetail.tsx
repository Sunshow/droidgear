import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Pencil, Trash2, AlertCircle, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
import { KeyList } from './KeyList'
import { useChannelStore } from '@/store/channel-store'
import { useModelStore } from '@/store/model-store'
import {
  commands,
  type Channel,
  type ChannelToken,
  type ChannelType,
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
import { getDefaultMaxOutputTokens } from '@/lib/utils'
import { BatchModelSelector } from '@/components/models/BatchModelSelector'
import { isBatchValid, type BatchModelConfig } from '@/lib/batch-model-utils'

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
  const [selectedModels, setSelectedModels] = useState<
    Map<string, BatchModelConfig>
  >(new Map())
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [modelError, setModelError] = useState<string | null>(null)
  const [prefix, setPrefix] = useState('')
  const [suffix, setSuffix] = useState('')
  const [batchMaxTokens, setBatchMaxTokens] = useState('')
  const [batchSupportsImages, setBatchSupportsImages] = useState(false)

  useEffect(() => {
    loadModels()
  }, [loadModels])

  const handleDelete = async () => {
    await deleteChannel(channel.id)
    await saveChannels()
    setDeleteDialogOpen(false)
  }

  const inferProvider = (modelId: string): Provider => {
    return channel.type === 'new-api'
      ? inferProviderForNewApi(modelId)
      : inferProviderFromPlatformAndModel(selectedKey?.platform, modelId)
  }

  const handleSelectKey = async (apiKey: ChannelToken) => {
    setSelectedKey(apiKey)
    setModelDialogOpen(true)
    setIsFetchingModels(true)
    setModelError(null)
    setSelectedModels(new Map())
    setPrefix('')
    setSuffix('')
    setBatchMaxTokens('')
    setBatchSupportsImages(false)

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
        next.set(modelId, { alias: '', provider: inferProvider(modelId) })
      }
      return next
    })
  }

  const handleConfigChange = (
    modelId: string,
    config: Partial<BatchModelConfig>
  ) => {
    setSelectedModels(prev => {
      const next = new Map(prev)
      const current = next.get(modelId)
      if (current) {
        next.set(modelId, { ...current, ...config })
      }
      return next
    })
  }

  const handleSelectAll = () => {
    const newMap = new Map<string, BatchModelConfig>()
    const selectableModels = availableModels.filter(
      m =>
        !existingModels.some(
          em => em.model === m.id && em.apiKey === selectedKey?.key
        )
    )
    selectableModels.forEach(m => {
      newMap.set(m.id, { alias: '', provider: inferProvider(m.id) })
    })
    setSelectedModels(newMap)
  }

  const handleDeselectAll = () => {
    setSelectedModels(new Map())
  }

  const handleAddModels = async () => {
    if (!selectedKey || selectedModels.size === 0) return

    for (const [modelId, config] of selectedModels) {
      if (
        existingModels.some(
          m => m.model === modelId && m.apiKey === selectedKey.key
        )
      ) {
        continue
      }

      const baseUrl =
        channel.type === 'new-api'
          ? getBaseUrlForNewApi(config.provider, channel.baseUrl)
          : getBaseUrlForSub2Api(
              config.provider,
              channel.baseUrl,
              selectedKey?.platform
            )

      let displayName = modelId
      if (config.alias) {
        displayName = config.alias
      } else if (prefix || suffix) {
        displayName = `${prefix}${modelId}${suffix}`
      }

      // Determine max tokens
      let maxOutputTokens: number | undefined
      if (config.maxTokens !== undefined) {
        maxOutputTokens = config.maxTokens
      } else if (batchMaxTokens) {
        maxOutputTokens = parseInt(batchMaxTokens)
      } else {
        maxOutputTokens = getDefaultMaxOutputTokens(modelId)
      }

      // Determine supports images
      let supportsImages: boolean | undefined
      if (config.supportsImages !== undefined) {
        supportsImages = config.supportsImages
      } else if (batchSupportsImages) {
        supportsImages = true
      }

      addModel({
        model: modelId,
        baseUrl,
        apiKey: selectedKey.key,
        provider: config.provider,
        displayName,
        maxOutputTokens,
        supportsImages: supportsImages || undefined,
      })
    }

    await saveModels()
    setModelDialogOpen(false)
  }

  const batchValid = isBatchValid(selectedModels, prefix, suffix)

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
          defaultHeight={680}
          minWidth={600}
          minHeight={500}
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
              <BatchModelSelector
                models={availableModels}
                apiKey={selectedKey?.key ?? ''}
                existingModels={existingModels}
                defaultProvider="anthropic"
                inferProvider={inferProvider}
                prefix={prefix}
                suffix={suffix}
                batchMaxTokens={batchMaxTokens}
                batchSupportsImages={batchSupportsImages}
                selectedModels={selectedModels}
                onPrefixChange={setPrefix}
                onSuffixChange={setSuffix}
                onBatchMaxTokensChange={setBatchMaxTokens}
                onBatchSupportsImagesChange={setBatchSupportsImages}
                onToggleModel={handleToggleModel}
                onConfigChange={handleConfigChange}
                onSelectAll={handleSelectAll}
                onDeselectAll={handleDeselectAll}
              />
            )}
          </ResizableDialogBody>

          <ResizableDialogFooter>
            <Button variant="outline" onClick={() => setModelDialogOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleAddModels}
              disabled={!batchValid || isFetchingModels}
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
