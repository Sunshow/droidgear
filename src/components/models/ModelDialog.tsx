import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, FolderInput } from 'lucide-react'
import {
  ResizableDialog,
  ResizableDialogContent,
  ResizableDialogHeader,
  ResizableDialogBody,
  ResizableDialogTitle,
  ResizableDialogFooter,
} from '@/components/ui/resizable-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  commands,
  type CustomModel,
  type Provider,
  type ModelInfo,
} from '@/lib/bindings'
import {
  containsRegexSpecialChars,
  getDefaultMaxOutputTokens,
  hasOfficialModelNamePrefix,
} from '@/lib/utils'
import { useModelStore } from '@/store/model-store'
import { BatchModelSelector } from './BatchModelSelector'
import {
  buildModelsFromBatch,
  isBatchValid,
  type BatchModelConfig,
} from '@/lib/batch-model-utils'
import { ChannelModelPickerDialog } from '@/components/channels/ChannelModelPickerDialog'

interface ModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  model?: CustomModel
  mode: 'add' | 'edit' | 'duplicate'
  onSave: (model: CustomModel) => void
  onSaveBatch?: (models: CustomModel[]) => void
}

const defaultBaseUrls: Record<Provider, string> = {
  anthropic: 'https://api.anthropic.com',
  openai: 'https://api.openai.com',
  'generic-chat-completion-api': '',
}

interface ModelFormProps {
  model?: CustomModel
  mode: 'add' | 'edit' | 'duplicate'
  onSave: (model: CustomModel) => void
  onSaveBatch?: (models: CustomModel[]) => void
  onCancel: () => void
}

function ModelForm({
  model,
  mode,
  onSave,
  onSaveBatch,
  onCancel,
}: ModelFormProps) {
  const { t } = useTranslation()
  const existingModels = useModelStore(state => state.models)

  const [provider, setProvider] = useState<Provider>(
    model?.provider ?? 'anthropic'
  )
  const [baseUrl, setBaseUrl] = useState(
    model?.baseUrl ?? defaultBaseUrls.anthropic
  )
  const [apiKey, setApiKey] = useState(model?.apiKey ?? '')
  const [modelId, setModelId] = useState(model?.model ?? '')
  const [displayName, setDisplayName] = useState(model?.displayName ?? '')
  const [maxTokens, setMaxTokens] = useState(
    model?.maxOutputTokens?.toString() ?? ''
  )
  const [supportsImages, setSupportsImages] = useState(
    model?.supportsImages ?? false
  )

  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([])
  const [isFetching, setIsFetching] = useState(false)
  const [fetchError, setFetchError] = useState<string | null>(null)

  // Batch mode state
  const [batchMode, setBatchMode] = useState(false)
  const [selectedModels, setSelectedModels] = useState<
    Map<string, BatchModelConfig>
  >(new Map())
  const [prefix, setPrefix] = useState('')
  const [suffix, setSuffix] = useState('')
  const [batchMaxTokens, setBatchMaxTokens] = useState('')
  const [batchSupportsImages, setBatchSupportsImages] = useState(false)

  // Channel picker state
  const [channelPickerOpen, setChannelPickerOpen] = useState(false)

  const handleModelIdChange = (newModelId: string) => {
    setModelId(newModelId)
    setDisplayName(newModelId)
    if (newModelId && !maxTokens) {
      setMaxTokens(getDefaultMaxOutputTokens(newModelId).toString())
    }
  }

  const handleProviderChange = (value: Provider) => {
    setProvider(value)
    setBaseUrl(current => current || defaultBaseUrls[value])
    setAvailableModels([])
    setFetchError(null)
    setBatchMode(false)
    setSelectedModels(new Map())
  }

  const handleFetchModels = async () => {
    if (!baseUrl || !apiKey) {
      setFetchError(t('models.fetchModelsError'))
      return
    }

    setIsFetching(true)
    setFetchError(null)

    const result = await commands.fetchModels(provider, baseUrl, apiKey)

    setIsFetching(false)

    if (result.status === 'ok') {
      setAvailableModels(result.data)
      if (result.data.length === 0) {
        setFetchError(t('models.noModelsFound'))
      } else if (result.data.length > 1 && mode !== 'edit' && onSaveBatch) {
        setBatchMode(true)
      }
    } else {
      setFetchError(result.error)
    }
  }

  const handleToggleModel = (modelIdToToggle: string) => {
    setSelectedModels(prev => {
      const next = new Map(prev)
      if (next.has(modelIdToToggle)) {
        next.delete(modelIdToToggle)
      } else {
        next.set(modelIdToToggle, { alias: '', provider })
      }
      return next
    })
  }

  const handleConfigChange = (
    modelIdToChange: string,
    config: Partial<BatchModelConfig>
  ) => {
    setSelectedModels(prev => {
      const next = new Map(prev)
      const current = next.get(modelIdToChange)
      if (current) {
        next.set(modelIdToChange, { ...current, ...config })
      }
      return next
    })
  }

  const handleSelectAll = () => {
    const newMap = new Map<string, BatchModelConfig>()
    const selectableModels = availableModels.filter(
      m =>
        !existingModels.some(
          em =>
            em.model === m.id && em.baseUrl === baseUrl && em.apiKey === apiKey
        )
    )
    selectableModels.forEach(m => {
      newMap.set(m.id, { alias: '', provider })
    })
    setSelectedModels(newMap)
  }

  const handleDeselectAll = () => {
    setSelectedModels(new Map())
  }

  const handleSave = () => {
    if (!modelId || !baseUrl || !apiKey) return

    const newModel: CustomModel = {
      model: modelId,
      baseUrl: baseUrl,
      apiKey: apiKey,
      provider,
      displayName: displayName || undefined,
      maxOutputTokens: maxTokens ? parseInt(maxTokens) : undefined,
      supportsImages: supportsImages || undefined,
    }

    onSave(newModel)
  }

  const handleSaveBatch = () => {
    if (!onSaveBatch || selectedModels.size === 0) return

    const models = buildModelsFromBatch(
      selectedModels,
      baseUrl,
      apiKey,
      prefix,
      suffix,
      batchMaxTokens,
      batchSupportsImages,
      existingModels
    )

    if (models.length > 0) {
      onSaveBatch(models)
    }
  }

  const handleImportFromChannel = (models: CustomModel[]) => {
    if (models.length > 0 && onSaveBatch) {
      onSaveBatch(models)
    }
  }

  const isValid =
    modelId &&
    baseUrl &&
    apiKey &&
    (!displayName ||
      (!containsRegexSpecialChars(displayName) &&
        !hasOfficialModelNamePrefix(displayName)))

  const batchValid = isBatchValid(selectedModels, prefix, suffix)

  return (
    <>
      <ResizableDialogBody>
        <div className="grid gap-4">
          {/* Import from Channel button - only show in add mode with batch support */}
          {mode === 'add' && onSaveBatch && (
            <Button
              variant="outline"
              className="w-full"
              onClick={() => setChannelPickerOpen(true)}
            >
              <FolderInput className="h-4 w-4 mr-2" />
              {t('channels.importFromChannel')}
            </Button>
          )}

          <div className="grid gap-2">
            <Label htmlFor="provider">{t('models.provider')}</Label>
            <Select value={provider} onValueChange={handleProviderChange}>
              <SelectTrigger>
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
          </div>

          <div className="grid gap-2">
            <Label htmlFor="baseUrl">{t('models.apiUrl')}</Label>
            <Input
              id="baseUrl"
              value={baseUrl}
              onChange={e => setBaseUrl(e.target.value)}
              placeholder="https://api.example.com"
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="apiKey">{t('models.apiKey')}</Label>
            <div className="flex gap-2">
              <Input
                id="apiKey"
                type="password"
                value={apiKey}
                onChange={e => setApiKey(e.target.value)}
                placeholder="sk-..."
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
                  t('models.fetchModels')
                )}
              </Button>
            </div>
            {fetchError && (
              <p className="text-sm text-destructive">{fetchError}</p>
            )}
          </div>

          {batchMode ? (
            <BatchModelSelector
              models={availableModels}
              apiKey={apiKey}
              existingModels={existingModels}
              defaultProvider={provider}
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
          ) : (
            <>
              <div className="grid gap-2">
                <Label htmlFor="model">{t('models.model')}</Label>
                {availableModels.length > 0 ? (
                  <Select value={modelId} onValueChange={handleModelIdChange}>
                    <SelectTrigger>
                      <SelectValue placeholder={t('models.selectModel')} />
                    </SelectTrigger>
                    <SelectContent>
                      {availableModels.map(m => (
                        <SelectItem key={m.id} value={m.id}>
                          {m.name || m.id}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                ) : (
                  <Input
                    id="model"
                    value={modelId}
                    onChange={e => handleModelIdChange(e.target.value)}
                    placeholder="claude-sonnet-4-5-20250929"
                  />
                )}
              </div>

              <div className="grid gap-2">
                <Label htmlFor="displayName">{t('models.displayName')}</Label>
                <Input
                  id="displayName"
                  value={displayName}
                  onChange={e => setDisplayName(e.target.value)}
                  placeholder="My Custom Model"
                />
                {containsRegexSpecialChars(displayName) && (
                  <p className="text-sm text-destructive">
                    {t('validation.bracketsNotAllowed')}
                  </p>
                )}
                {hasOfficialModelNamePrefix(displayName) && (
                  <p className="text-sm text-destructive">
                    {t('validation.officialModelNameNotAllowed')}
                  </p>
                )}
              </div>

              <div className="grid gap-2">
                <Label htmlFor="maxTokens">{t('models.maxTokens')}</Label>
                <Input
                  id="maxTokens"
                  type="number"
                  value={maxTokens}
                  onChange={e => setMaxTokens(e.target.value)}
                  placeholder="8192"
                  step={8192}
                />
              </div>

              <div className="flex items-center gap-2">
                <Checkbox
                  id="supportsImages"
                  checked={supportsImages}
                  onCheckedChange={checked =>
                    setSupportsImages(checked === true)
                  }
                />
                <Label htmlFor="supportsImages">
                  {t('models.supportsImages')}
                </Label>
              </div>
            </>
          )}
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        {batchMode ? (
          <Button onClick={handleSaveBatch} disabled={!batchValid}>
            {selectedModels.size === 1
              ? t('models.addCount', { count: selectedModels.size })
              : t('models.addCountPlural', { count: selectedModels.size })}
          </Button>
        ) : (
          <Button onClick={handleSave} disabled={!isValid}>
            {model ? t('common.save') : t('common.add')}
          </Button>
        )}
      </ResizableDialogFooter>

      {/* Channel Model Picker Dialog */}
      <ChannelModelPickerDialog
        open={channelPickerOpen}
        onOpenChange={setChannelPickerOpen}
        mode="multiple"
        existingModels={existingModels}
        onSelect={handleImportFromChannel}
        showBatchConfig={true}
      />
    </>
  )
}

export function ModelDialog({
  open,
  onOpenChange,
  model,
  mode,
  onSave,
  onSaveBatch,
}: ModelDialogProps) {
  const { t } = useTranslation()
  const formKey = model ? `edit-${model.model}` : 'new'

  const handleSave = (newModel: CustomModel) => {
    onSave(newModel)
    onOpenChange(false)
  }

  const handleSaveBatch = (models: CustomModel[]) => {
    onSaveBatch?.(models)
    onOpenChange(false)
  }

  const titleKey =
    mode === 'edit'
      ? 'models.editModel'
      : mode === 'duplicate'
        ? 'models.duplicateModel'
        : 'models.addModel'

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={700}
        defaultHeight={680}
        minWidth={600}
        minHeight={500}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>{t(titleKey)}</ResizableDialogTitle>
        </ResizableDialogHeader>
        {open && (
          <ModelForm
            key={formKey}
            model={model}
            mode={mode}
            onSave={handleSave}
            onSaveBatch={onSaveBatch ? handleSaveBatch : undefined}
            onCancel={() => onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
