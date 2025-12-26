import { useState, useEffect } from 'react'
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
  type ModelInfo,
  type CustomModel,
} from '@/lib/bindings'

interface ChannelDetailProps {
  channel: Channel
  onEdit: () => void
}

export function ChannelDetail({ channel, onEdit }: ChannelDetailProps) {
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
            {!channel.enabled && <Badge variant="secondary">Disabled</Badge>}
          </div>
          <p className="text-sm text-muted-foreground mt-1">
            {channel.type === 'new-api' ? 'New API' : 'One API'} -{' '}
            {channel.baseUrl}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={onEdit}>
            <Pencil className="h-4 w-4 mr-2" />
            Edit
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setDeleteDialogOpen(true)}
          >
            <Trash2 className="h-4 w-4 mr-2" />
            Delete
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
            Dismiss
          </Button>
        </div>
      )}

      {/* Token List */}
      <div className="flex-1 overflow-auto p-4">
        <TokenList
          channelId={channel.id}
          baseUrl={channel.baseUrl}
          onSelectToken={handleSelectToken}
        />
      </div>

      {/* Delete Confirmation */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Channel</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete &quot;{channel.name}&quot;? This
              will also remove the system token from your keychain. This action
              cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>Delete</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Model Selection Dialog */}
      <Dialog open={modelDialogOpen} onOpenChange={setModelDialogOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Add Models from Token</DialogTitle>
          </DialogHeader>

          {isFetchingModels ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin mr-2" />
              <span>Fetching available models...</span>
            </div>
          ) : modelError ? (
            <div className="py-4 text-center text-destructive">
              <p>{modelError}</p>
            </div>
          ) : availableModels.length === 0 ? (
            <div className="py-4 text-center text-muted-foreground">
              <p>No models available for this token.</p>
            </div>
          ) : (
            <div className="py-4 space-y-4">
              <div className="flex items-center justify-between">
                <Label>
                  Select models to add ({selectedModels.size} selected)
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
                    ? 'Deselect All'
                    : 'Select All'}
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
                              (already added)
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
              Cancel
            </Button>
            <Button
              onClick={handleAddModels}
              disabled={selectedModels.size === 0 || isFetchingModels}
            >
              Add {selectedModels.size} Model
              {selectedModels.size !== 1 ? 's' : ''}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
