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
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { TokenList } from './TokenList'
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
  const [selectedToken, setSelectedToken] = useState<ChannelToken | null>(null)
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([])
  const [selectedModels, setSelectedModels] = useState<Set<string>>(new Set())
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [modelError, setModelError] = useState<string | null>(null)

  // Load models on mount
  useEffect(() => {
    loadModels()
  }, [loadModels])

  const handleDelete = async () => {
    await deleteChannel(channel.id)
    await saveChannels()
    setDeleteDialogOpen(false)
  }

  const handleSelectToken = async (token: ChannelToken) => {
    setSelectedToken(token)
    setModelDialogOpen(true)
    setIsFetchingModels(true)
    setModelError(null)
    setSelectedModels(new Set())

    const result = await commands.fetchModelsByToken(channel.baseUrl, token.key)
    setIsFetchingModels(false)

    if (result.status === 'ok') {
      setAvailableModels(result.data)
    } else {
      setModelError(result.error)
    }
  }

  const handleToggleModel = (modelId: string) => {
    setSelectedModels(prev => {
      const next = new Set(prev)
      if (next.has(modelId)) {
        next.delete(modelId)
      } else {
        next.add(modelId)
      }
      return next
    })
  }

  const handleAddModels = async () => {
    if (!selectedToken || selectedModels.size === 0) return

    const existingModelIds = new Set(existingModels.map(m => m.model))

    for (const modelId of selectedModels) {
      if (existingModelIds.has(modelId)) continue

      const newModel: CustomModel = {
        model: modelId,
        baseUrl: channel.baseUrl,
        apiKey: selectedToken.key,
        provider: 'generic-chat-completion-api',
        displayName: modelId,
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

      {/* Token List */}
      <div className="flex-1 overflow-auto p-4">
        <TokenList
          channelId={channel.id}
          channelType={channel.type}
          baseUrl={channel.baseUrl}
          onSelectToken={handleSelectToken}
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
                    if (selectedModels.size === availableModels.length) {
                      setSelectedModels(new Set())
                    } else {
                      setSelectedModels(new Set(availableModels.map(m => m.id)))
                    }
                  }}
                >
                  {selectedModels.size === availableModels.length
                    ? t('common.deselectAll')
                    : t('common.selectAll')}
                </Button>
              </div>
              <div className="h-[300px] border rounded-md p-2 overflow-auto">
                <div className="space-y-2">
                  {availableModels.map(model => {
                    const isExisting = existingModels.some(
                      m => m.model === model.id
                    )
                    return (
                      <div
                        key={model.id}
                        className="flex items-center gap-2 p-2 rounded hover:bg-accent/50"
                      >
                        <Checkbox
                          id={model.id}
                          checked={selectedModels.has(model.id)}
                          onCheckedChange={() => handleToggleModel(model.id)}
                          disabled={isExisting}
                        />
                        <label
                          htmlFor={model.id}
                          className="flex-1 text-sm cursor-pointer"
                        >
                          {model.name || model.id}
                          {isExisting && (
                            <span className="ml-2 text-xs text-muted-foreground">
                              {t('models.alreadyAdded')}
                            </span>
                          )}
                        </label>
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
