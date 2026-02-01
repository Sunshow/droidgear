import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Trash2, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  ResizableDialog,
  ResizableDialogContent,
  ResizableDialogDescription,
  ResizableDialogFooter,
  ResizableDialogHeader,
  ResizableDialogBody,
  ResizableDialogTitle,
} from '@/components/ui/resizable-dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useOpenClawStore } from '@/store/openclaw-store'
import {
  commands,
  type OpenClawProfile,
  type OpenClawModel,
  type ModelInfo,
} from '@/lib/bindings'

interface ProviderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingProviderId: string | null
  currentProfile: OpenClawProfile | null
}

const API_TYPES = [
  { value: 'openai-completions', label: 'OpenAI Completions' },
  { value: 'anthropic-messages', label: 'Anthropic Messages' },
]

interface ProviderFormProps {
  editingProviderId: string | null
  currentProfile: OpenClawProfile | null
  onClose: () => void
}

function ProviderForm({
  editingProviderId,
  currentProfile,
  onClose,
}: ProviderFormProps) {
  const { t } = useTranslation()
  const addProvider = useOpenClawStore(state => state.addProvider)
  const updateProvider = useOpenClawStore(state => state.updateProvider)

  const isEditing = editingProviderId !== null
  const existingConfig = editingProviderId
    ? currentProfile?.providers[editingProviderId]
    : null

  const [providerId, setProviderId] = useState(editingProviderId ?? '')
  const [baseUrl, setBaseUrl] = useState(existingConfig?.baseUrl ?? '')
  const [apiKey, setApiKey] = useState(existingConfig?.apiKey ?? '')
  const [api, setApi] = useState(existingConfig?.api ?? 'openai-completions')
  const [models, setModels] = useState<OpenClawModel[]>(
    existingConfig?.models ?? []
  )

  // Fetch models state
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([])
  const [selectedModelIds, setSelectedModelIds] = useState<Set<string>>(
    new Set()
  )
  const [isFetching, setIsFetching] = useState(false)
  const [fetchError, setFetchError] = useState<string | null>(null)

  const handleAddModel = () => {
    setModels([
      ...models,
      {
        id: '',
        name: null,
        reasoning: false,
        input: ['text'],
        contextWindow: null,
        maxTokens: null,
      },
    ])
  }

  const handleFetchModels = async () => {
    if (!baseUrl || !apiKey) {
      setFetchError(t('openclaw.provider.fetchModelsError'))
      return
    }

    setIsFetching(true)
    setFetchError(null)

    // Map API type to provider
    const provider = api === 'anthropic-messages' ? 'anthropic' : 'openai'

    const result = await commands.fetchModels(provider, baseUrl, apiKey)

    setIsFetching(false)

    if (result.status === 'ok') {
      setAvailableModels(result.data)
      if (result.data.length === 0) {
        setFetchError(t('openclaw.provider.noModelsFound'))
      }
    } else {
      setFetchError(result.error)
    }
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

  const handleAddSelectedModels = () => {
    const newModels: OpenClawModel[] = Array.from(selectedModelIds).map(id => {
      const info = availableModels.find(m => m.id === id)
      return {
        id,
        name: info?.name ?? null,
        reasoning: false,
        input: ['text'],
        contextWindow: null,
        maxTokens: null,
      }
    })
    // Merge with existing models, avoiding duplicates
    const existingIds = new Set(models.map(m => m.id))
    const uniqueNewModels = newModels.filter(m => !existingIds.has(m.id))
    setModels([...models, ...uniqueNewModels])
    setSelectedModelIds(new Set())
  }

  const handleRemoveModel = (index: number) => {
    setModels(models.filter((_, i) => i !== index))
  }

  const handleModelChange = (
    index: number,
    field: keyof OpenClawModel,
    value: string | boolean | string[] | number | null
  ) => {
    const updated = [...models]
    const model = updated[index]
    if (!model) return
    updated[index] = { ...model, [field]: value }
    setModels(updated)
  }

  const handleSave = () => {
    if (!providerId.trim()) return

    const config = {
      baseUrl: baseUrl.trim() || null,
      apiKey: apiKey.trim() || null,
      api: api || null,
      models: models.filter(m => m.id.trim()),
    }

    if (isEditing) {
      updateProvider(providerId, config)
    } else {
      addProvider(providerId.trim(), config)
    }

    onClose()
  }

  return (
    <>
      <ResizableDialogBody>
        <div className="space-y-4">
          {/* Provider ID */}
          <div className="space-y-2">
            <Label>{t('openclaw.provider.id')} *</Label>
            <Input
              value={providerId}
              onChange={e => setProviderId(e.target.value)}
              placeholder="custom-provider"
              disabled={isEditing}
            />
          </div>

          {/* Base URL */}
          <div className="space-y-2">
            <Label>{t('openclaw.provider.baseUrl')} *</Label>
            <Input
              value={baseUrl}
              onChange={e => setBaseUrl(e.target.value)}
              placeholder="https://api.example.com/v1"
            />
          </div>

          {/* API Key */}
          <div className="space-y-2">
            <Label>{t('openclaw.provider.apiKey')}</Label>
            <div className="flex gap-2">
              <Input
                type="password"
                value={apiKey}
                onChange={e => setApiKey(e.target.value)}
                placeholder="${API_KEY}"
                className="flex-1"
              />
              <Button
                variant="outline"
                onClick={handleFetchModels}
                disabled={isFetching || !baseUrl || !apiKey}
              >
                {isFetching ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  t('openclaw.provider.fetchModels')
                )}
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              {t('openclaw.provider.apiKeyHint')}
            </p>
            {fetchError && (
              <p className="text-sm text-destructive">{fetchError}</p>
            )}
          </div>

          {/* API Type */}
          <div className="space-y-2">
            <Label>{t('openclaw.provider.apiType')}</Label>
            <Select value={api} onValueChange={setApi}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {API_TYPES.map(type => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Fetched Models Selection */}
          {availableModels.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>{t('openclaw.provider.availableModels')}</Label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={handleAddSelectedModels}
                  disabled={selectedModelIds.size === 0}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  {t('openclaw.provider.addSelected')} ({selectedModelIds.size})
                </Button>
              </div>
              <div className="max-h-40 overflow-y-auto border rounded-lg p-2 space-y-1">
                {availableModels.map(m => {
                  const alreadyAdded = models.some(em => em.id === m.id)
                  return (
                    <div
                      key={m.id}
                      className="flex items-center gap-2 py-1 px-2 hover:bg-muted/50 rounded"
                    >
                      <Checkbox
                        id={`model-${m.id}`}
                        checked={selectedModelIds.has(m.id)}
                        onCheckedChange={() => handleToggleModel(m.id)}
                        disabled={alreadyAdded}
                      />
                      <label
                        htmlFor={`model-${m.id}`}
                        className={`text-sm flex-1 cursor-pointer ${alreadyAdded ? 'text-muted-foreground' : ''}`}
                      >
                        {m.name || m.id}
                        {alreadyAdded && (
                          <span className="ml-2 text-xs text-muted-foreground">
                            ({t('openclaw.provider.alreadyAdded')})
                          </span>
                        )}
                      </label>
                    </div>
                  )
                })}
              </div>
            </div>
          )}

          {/* Models */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>{t('openclaw.provider.models')}</Label>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleAddModel}
              >
                <Plus className="h-4 w-4 mr-1" />
                {t('openclaw.provider.addModel')}
              </Button>
            </div>

            {models.length === 0 ? (
              <p className="text-sm text-muted-foreground py-2">
                {t('openclaw.provider.noModels')}
              </p>
            ) : (
              <div className="space-y-3">
                {models.map((model, index) => (
                  <div key={index} className="p-3 border rounded-lg space-y-2">
                    <div className="flex items-center justify-between">
                      <Label className="text-sm">
                        {t('openclaw.provider.model')} #{index + 1}
                      </Label>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRemoveModel(index)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      <div>
                        <Label className="text-xs">
                          {t('openclaw.provider.modelId')} *
                        </Label>
                        <Input
                          value={model.id}
                          onChange={e =>
                            handleModelChange(index, 'id', e.target.value)
                          }
                          placeholder="model-id"
                          className="h-8"
                        />
                      </div>
                      <div>
                        <Label className="text-xs">
                          {t('openclaw.provider.modelName')}
                        </Label>
                        <Input
                          value={model.name ?? ''}
                          onChange={e =>
                            handleModelChange(
                              index,
                              'name',
                              e.target.value || null
                            )
                          }
                          placeholder="Display Name"
                          className="h-8"
                        />
                      </div>
                      <div>
                        <Label className="text-xs">
                          {t('openclaw.provider.contextWindow')}
                        </Label>
                        <Input
                          type="number"
                          value={model.contextWindow ?? ''}
                          onChange={e =>
                            handleModelChange(
                              index,
                              'contextWindow',
                              e.target.value
                                ? parseInt(e.target.value, 10)
                                : null
                            )
                          }
                          placeholder="200000"
                          className="h-8"
                        />
                      </div>
                      <div>
                        <Label className="text-xs">
                          {t('openclaw.provider.maxTokens')}
                        </Label>
                        <Input
                          type="number"
                          value={model.maxTokens ?? ''}
                          onChange={e =>
                            handleModelChange(
                              index,
                              'maxTokens',
                              e.target.value
                                ? parseInt(e.target.value, 10)
                                : null
                            )
                          }
                          placeholder="8192"
                          className="h-8"
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button onClick={handleSave} disabled={!providerId.trim()}>
          {t('common.save')}
        </Button>
      </ResizableDialogFooter>
    </>
  )
}

export function ProviderDialog({
  open,
  onOpenChange,
  editingProviderId,
  currentProfile,
}: ProviderDialogProps) {
  const { t } = useTranslation()
  const isEditing = editingProviderId !== null
  const formKey = editingProviderId ?? 'new'

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={650}
        defaultHeight={600}
        minWidth={500}
        minHeight={400}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>
            {isEditing
              ? t('openclaw.provider.edit')
              : t('openclaw.provider.add')}
          </ResizableDialogTitle>
          <ResizableDialogDescription>
            {t('openclaw.provider.dialogDescription')}
          </ResizableDialogDescription>
        </ResizableDialogHeader>
        {open && (
          <ProviderForm
            key={formKey}
            editingProviderId={editingProviderId}
            currentProfile={currentProfile}
            onClose={() => onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
