import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, FolderInput, ChevronDown, ChevronRight } from 'lucide-react'
import { Textarea } from '@/components/ui/textarea'
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
import { SecretInput } from '@/components/ui/secret-input'
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
  type JsonValue,
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
  const [noImageSupport, setNoImageSupport] = useState(
    model?.noImageSupport ?? false
  )
  // Extract reasoning effort from extraArgs if present
  const extractReasoningEffort = (
    args?: Partial<Record<string, JsonValue>> | null
  ): string => {
    if (!args) return 'none'
    const reasoning = args.reasoning
    if (
      reasoning &&
      typeof reasoning === 'object' &&
      !Array.isArray(reasoning) &&
      reasoning !== null
    ) {
      const effort = (reasoning as Record<string, JsonValue>).effort
      if (typeof effort === 'string') return effort
    }
    return 'none'
  }

  const [reasoningEffort, setReasoningEffort] = useState(
    extractReasoningEffort(model?.extraArgs)
  )
  const [extraArgs, setExtraArgs] = useState(
    model?.extraArgs ? JSON.stringify(model.extraArgs, null, 2) : ''
  )
  const [extraHeaders, setExtraHeaders] = useState(
    model?.extraHeaders ? JSON.stringify(model.extraHeaders, null, 2) : ''
  )
  const [showAdvanced, setShowAdvanced] = useState(
    !!(model?.extraArgs || model?.extraHeaders)
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
  const [batchNoImageSupport, setBatchNoImageSupport] = useState(false)

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

  const parseJsonSafe = (
    value: string
  ): Partial<Record<string, JsonValue>> | undefined => {
    const trimmed = value.trim()
    if (!trimmed) return undefined
    try {
      const parsed = JSON.parse(trimmed)
      if (
        typeof parsed === 'object' &&
        parsed !== null &&
        !Array.isArray(parsed)
      ) {
        return parsed as Partial<Record<string, JsonValue>>
      }
      return undefined
    } catch {
      return undefined
    }
  }

  const isJsonValid = (value: string): boolean => {
    const trimmed = value.trim()
    if (!trimmed) return true
    try {
      const parsed = JSON.parse(trimmed)
      return (
        typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)
      )
    } catch {
      return false
    }
  }

  const extraArgsValid = isJsonValid(extraArgs)
  const extraHeadersValid = isJsonValid(extraHeaders)

  const handleSave = () => {
    if (!modelId || !baseUrl || !apiKey) return
    if (!extraArgsValid || !extraHeadersValid) return

    const newModel: CustomModel = {
      model: modelId,
      baseUrl: baseUrl,
      apiKey: apiKey,
      provider,
      displayName: displayName || undefined,
      maxOutputTokens: maxTokens ? parseInt(maxTokens) : undefined,
      noImageSupport: noImageSupport || undefined,
      extraArgs: (() => {
        const parsed = parseJsonSafe(extraArgs) ?? {}
        if (reasoningEffort && reasoningEffort !== 'none') {
          parsed.reasoning = { effort: reasoningEffort }
        } else {
          delete parsed.reasoning
        }
        return Object.keys(parsed).length > 0 ? parsed : undefined
      })(),
      extraHeaders: parseJsonSafe(extraHeaders) as
        | Record<string, string>
        | null
        | undefined,
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
      batchNoImageSupport,
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
    extraArgsValid &&
    extraHeadersValid &&
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
              <SecretInput
                id="apiKey"
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
              batchNoImageSupport={batchNoImageSupport}
              selectedModels={selectedModels}
              onPrefixChange={setPrefix}
              onSuffixChange={setSuffix}
              onBatchMaxTokensChange={setBatchMaxTokens}
              onBatchNoImageSupportChange={setBatchNoImageSupport}
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
                  id="noImageSupport"
                  checked={noImageSupport}
                  onCheckedChange={checked =>
                    setNoImageSupport(checked === true)
                  }
                />
                <Label htmlFor="noImageSupport">
                  {t('models.noImageSupport')}
                </Label>
              </div>

              {/* Reasoning Effort */}
              <div className="grid gap-2">
                <Label htmlFor="reasoningEffort">
                  {t('models.reasoningEffort')}
                </Label>
                <Select
                  value={reasoningEffort}
                  onValueChange={setReasoningEffort}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">
                      {t('models.reasoningEffort.none')}
                    </SelectItem>
                    <SelectItem value="low">
                      {t('models.reasoningEffort.low')}
                    </SelectItem>
                    <SelectItem value="medium">
                      {t('models.reasoningEffort.medium')}
                    </SelectItem>
                    <SelectItem value="high">
                      {t('models.reasoningEffort.high')}
                    </SelectItem>
                    <SelectItem value="xhigh">
                      {t('models.reasoningEffort.xhigh')}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {t('models.reasoningEffortHint')}
                </p>
              </div>

              {/* Advanced Options (extraArgs / extraHeaders) */}
              <button
                type="button"
                className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
                onClick={() => setShowAdvanced(!showAdvanced)}
              >
                {showAdvanced ? (
                  <ChevronDown className="h-4 w-4" />
                ) : (
                  <ChevronRight className="h-4 w-4" />
                )}
                {t('models.advancedOptions')}
              </button>

              {showAdvanced && (
                <div className="grid gap-3 pl-2 border-l-2 border-muted">
                  <div className="grid gap-2">
                    <Label htmlFor="extraArgs">{t('models.extraArgs')}</Label>
                    <Textarea
                      id="extraArgs"
                      value={extraArgs}
                      onChange={e => setExtraArgs(e.target.value)}
                      placeholder={t('models.extraArgsPlaceholder')}
                      rows={3}
                      className="font-mono text-sm"
                    />
                    {!extraArgsValid && (
                      <p className="text-sm text-destructive">
                        {t('models.invalidJson')}
                      </p>
                    )}
                    <p className="text-xs text-muted-foreground">
                      {t('models.extraArgsHint')}
                    </p>
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="extraHeaders">
                      {t('models.extraHeaders')}
                    </Label>
                    <Textarea
                      id="extraHeaders"
                      value={extraHeaders}
                      onChange={e => setExtraHeaders(e.target.value)}
                      placeholder={t('models.extraHeadersPlaceholder')}
                      rows={3}
                      className="font-mono text-sm"
                    />
                    {!extraHeadersValid && (
                      <p className="text-sm text-destructive">
                        {t('models.invalidJson')}
                      </p>
                    )}
                    <p className="text-xs text-muted-foreground">
                      {t('models.extraHeadersHint')}
                    </p>
                  </div>
                </div>
              )}
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
