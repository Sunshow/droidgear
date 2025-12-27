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
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
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
} from '@/lib/bindings'
import { getProviderConfigFromPlatform } from '@/lib/sub2api-platform'

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
  // Map of modelId -> alias (empty string means no custom alias)
  const [selectedModels, setSelectedModels] = useState<Map<string, string>>(
    new Map()
  )
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
        next.set(modelId, '') // Empty string means use default (prefix + modelId + suffix)
      }
      return next
    })
  }

  const handleAliasChange = (modelId: string, alias: string) => {
    setSelectedModels(prev => {
      const next = new Map(prev)
      next.set(modelId, alias)
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

    for (const [modelId, customAlias] of selectedModels) {
      // Skip if this model+key combination already exists
      if (isModelKeyExisting(modelId, selectedKey.key)) continue

      const providerConfig = getProviderConfigFromPlatform(
        selectedKey.platform,
        channel.baseUrl
      )

      // Determine display name: custom alias > prefix+model+suffix > model
      let displayName = modelId
      if (customAlias) {
        displayName = customAlias
      } else if (prefix || suffix) {
        displayName = `${prefix}${modelId}${suffix}`
      }

      const newModel: CustomModel = {
        model: modelId,
        baseUrl: providerConfig.baseUrl,
        apiKey: selectedKey.key,
        provider: providerConfig.provider,
        displayName,
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
      <Dialog open={modelDialogOpen} onOpenChange={setModelDialogOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>{t('models.addModels')}</DialogTitle>
          </DialogHeader>

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
            <div className="py-4 space-y-4">
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
                </div>
                <div className="space-y-2">
                  <Label htmlFor="suffix">{t('models.suffix')}</Label>
                  <Input
                    id="suffix"
                    value={suffix}
                    onChange={e => setSuffix(e.target.value)}
                    placeholder={t('models.suffixPlaceholder')}
                  />
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
                      const newMap = new Map<string, string>()
                      selectableModels.forEach(m => newMap.set(m.id, ''))
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
              <div className="h-[300px] border rounded-md p-2 overflow-auto">
                <div className="space-y-2">
                  {availableModels.map(model => {
                    const isExisting = isModelKeyExisting(
                      model.id,
                      selectedKey?.key ?? ''
                    )
                    const isSelected = selectedModels.has(model.id)
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
                          <Input
                            className="h-7 text-sm flex-1"
                            value={selectedModels.get(model.id) ?? ''}
                            onChange={e =>
                              handleAliasChange(model.id, e.target.value)
                            }
                            placeholder={t('models.aliasPlaceholder')}
                          />
                        )}
                      </div>
                    )
                  })}
                </div>
              </div>
            </div>
          )}

          <DialogFooter>
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
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
