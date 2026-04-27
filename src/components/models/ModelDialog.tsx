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
  effortToBudgetTokens,
  getDefaultMaxOutputTokens,
  hasOfficialModelNamePrefix,
  isAnthropicAdaptiveThinkingModel,
  isOpus47,
  supportsMaxEffort,
  supportsXhighEffort,
  type ReasoningEffort,
} from '@/lib/utils'
import {
  getSupportedEfforts,
  getEffortEncoding,
  getModelReasoningConfig,
} from '@/lib/model-registry'
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

const ANTHROPIC_BETA_1M_VALUE =
  'claude-code-20250219,context-1m-2025-08-07,interleaved-thinking-2025-05-14,redact-thinking-2026-02-12,context-management-2025-06-27,prompt-caching-scope-2026-01-05,effort-2025-11-24'

function has1MContextHeader(
  headers: Record<string, unknown> | null | undefined
): boolean {
  if (!headers || typeof headers !== 'object') return false
  return (
    (headers as Record<string, string>)['Anthropic-Beta'] ===
    ANTHROPIC_BETA_1M_VALUE
  )
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
  // Extract reasoning effort from extraArgs if present.
  // Supports:
  //   - reasoning.effort (OpenAI / generic)
  //   - reasoning_effort top-level key (OpenAI format for some models like DeepSeek)
  //   - output_config.effort paired with thinking.type (Anthropic adaptive & non-adaptive)
  const extractReasoningEffort = (
    args?: Partial<Record<string, JsonValue>> | null,
    modelIdForLookup?: string,
    providerForLookup?: Provider
  ): string => {
    if (!args) return 'none'

    // Registry reverse-mapping: check which encoding matches the current extraArgs
    if (modelIdForLookup && providerForLookup) {
      const config = getModelReasoningConfig(modelIdForLookup)
      if (config) {
        for (const effort of config.efforts) {
          const encoding = getEffortEncoding(
            modelIdForLookup,
            providerForLookup,
            effort
          )
          if (encoding) {
            const keys = Object.keys(encoding)
            const match = keys.every(k => {
              const a = args[k]
              const b = encoding[k]
              return JSON.stringify(a) === JSON.stringify(b)
            })
            if (match) return effort
          }
        }
      }
    }

    const thinking = args.thinking
    const thinkingEnabled =
      thinking &&
      typeof thinking === 'object' &&
      !Array.isArray(thinking) &&
      ((thinking as Record<string, JsonValue>).type === 'adaptive' ||
        (thinking as Record<string, JsonValue>).type === 'enabled')
    if (thinkingEnabled) {
      const outputConfig = args.output_config
      if (
        outputConfig &&
        typeof outputConfig === 'object' &&
        !Array.isArray(outputConfig)
      ) {
        const effort = (outputConfig as Record<string, JsonValue>).effort
        if (typeof effort === 'string') return effort
      }
    }
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
    // OpenAI format: reasoning_effort top-level key (used by DeepSeek)
    const reasoningEffort = args.reasoning_effort
    if (typeof reasoningEffort === 'string') return reasoningEffort
    return 'none'
  }

  // If the currently selected effort isn't supported by a model, snap it down
  // to the highest supported level. xhigh -> high, max -> xhigh -> high.
  const clampEffortToModel = (effort: string, nextModelId: string): string => {
    const supported = getSupportedEfforts(nextModelId, provider)
    // Registry whitelist takes priority
    if (supported) {
      if (supported.includes(effort as ReasoningEffort)) return effort
      // Find the highest supported effort to clamp to
      const effortOrder: ReasoningEffort[] = [
        'none',
        'low',
        'medium',
        'high',
        'xhigh',
        'max',
      ]
      const idx = effortOrder.indexOf(effort as ReasoningEffort)
      for (let i = idx - 1; i >= 0; i--) {
        const candidate = effortOrder[i]
        if (candidate && supported.includes(candidate)) return candidate
      }
      return supported[0] ?? 'high'
    }
    // Fallback to old logic
    if (effort === 'max' && !supportsMaxEffort(nextModelId)) {
      return supportsXhighEffort(nextModelId) ? 'xhigh' : 'high'
    }
    if (effort === 'xhigh' && !supportsXhighEffort(nextModelId)) {
      return 'high'
    }
    return effort
  }

  const [reasoningEffort, setReasoningEffort] = useState(() => {
    const extracted = extractReasoningEffort(
      model?.extraArgs,
      model?.model,
      provider
    )
    const modelIdForClamp = model?.model ?? ''
    return clampEffortToModel(extracted, modelIdForClamp)
  })
  // Track whether maxTokens was auto-filled vs user-edited, so effort changes
  // can re-fill only when the user hasn't manually overridden the value.
  const [autoFilledMaxTokens, setAutoFilledMaxTokens] = useState(
    !model?.maxOutputTokens
  )
  const [context1MSupport, setContext1MSupport] = useState(
    () => model?.extraHeaders != null && has1MContextHeader(model.extraHeaders)
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

  // Rewrite the effort-encoding keys in an extraArgs JSON string to match the
  // (provider, model, effort) triple, preserving unrelated fields. Also strips
  // sampling params that Opus 4.7 rejects. Returns the JSON unchanged when the
  // user has typed invalid JSON so we don't destroy in-progress edits.
  const rewriteExtraArgsWithEffort = (
    currentJson: string,
    nextProvider: Provider,
    nextModelId: string,
    nextEffort: string
  ): string => {
    if (currentJson.trim() && !isJsonValid(currentJson)) return currentJson
    const parsed = parseJsonSafe(currentJson) ?? {}
    delete parsed.reasoning
    delete parsed.reasoning_effort
    delete parsed.thinking
    delete parsed.output_config

    // Registry whitelist takes priority
    const encoding = getEffortEncoding(nextModelId, nextProvider, nextEffort)
    if (encoding) {
      Object.assign(parsed, encoding)
      if (isOpus47(nextModelId)) {
        delete parsed.temperature
        delete parsed.top_p
        delete parsed.top_k
      }
      return Object.keys(parsed).length > 0
        ? JSON.stringify(parsed, null, 2)
        : ''
    }

    // Fallback to old logic
    if (nextEffort && nextEffort !== 'none') {
      if (
        nextProvider === 'anthropic' &&
        isAnthropicAdaptiveThinkingModel(nextModelId)
      ) {
        parsed.thinking = { type: 'adaptive' }
        parsed.output_config = { effort: nextEffort }
      } else if (
        nextProvider === 'anthropic' &&
        nextModelId.startsWith('claude-')
      ) {
        parsed.thinking = {
          type: 'enabled',
          budget_tokens: effortToBudgetTokens(nextEffort),
        }
      } else if (nextProvider === 'anthropic') {
        parsed.thinking = { type: 'enabled' }
        parsed.output_config = { effort: nextEffort }
      } else {
        parsed.reasoning = { effort: nextEffort }
      }
    } else if (
      nextProvider === 'anthropic' &&
      !nextModelId.startsWith('claude-') &&
      nextModelId
    ) {
      parsed.thinking = { type: 'disabled' }
    }
    if (isOpus47(nextModelId)) {
      delete parsed.temperature
      delete parsed.top_p
      delete parsed.top_k
    }
    return Object.keys(parsed).length > 0 ? JSON.stringify(parsed, null, 2) : ''
  }

  const handleModelIdChange = (newModelId: string) => {
    setModelId(newModelId)
    setDisplayName(newModelId)
    const clampedEffort = clampEffortToModel(reasoningEffort, newModelId)
    if (clampedEffort !== reasoningEffort) setReasoningEffort(clampedEffort)
    setExtraArgs(
      rewriteExtraArgsWithEffort(extraArgs, provider, newModelId, clampedEffort)
    )
    if (newModelId && (!maxTokens || autoFilledMaxTokens)) {
      setMaxTokens(
        getDefaultMaxOutputTokens(newModelId, clampedEffort).toString()
      )
      setAutoFilledMaxTokens(true)
    }
  }

  const handleMaxTokensChange = (value: string) => {
    setMaxTokens(value)
    setAutoFilledMaxTokens(false)
  }

  const handleReasoningEffortChange = (value: string) => {
    setReasoningEffort(value)
    setExtraArgs(
      rewriteExtraArgsWithEffort(extraArgs, provider, modelId, value)
    )
    if (modelId && autoFilledMaxTokens) {
      setMaxTokens(getDefaultMaxOutputTokens(modelId, value).toString())
    }
  }

  const handleProviderChange = (value: Provider) => {
    setProvider(value)
    setBaseUrl(current => current || defaultBaseUrls[value])
    setAvailableModels([])
    setFetchError(null)
    setBatchMode(false)
    setSelectedModels(new Map())
    const clampedEffort = clampEffortToModel(reasoningEffort, modelId)
    if (clampedEffort !== reasoningEffort) setReasoningEffort(clampedEffort)
    setExtraArgs(
      rewriteExtraArgsWithEffort(extraArgs, value, modelId, clampedEffort)
    )
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

  const validateJson = (
    value: string
  ):
    | { ok: true; hasSmartQuotes: false }
    | { ok: false; error: string; hasSmartQuotes: boolean } => {
    const trimmed = value.trim()
    if (!trimmed) return { ok: true, hasSmartQuotes: false }
    // U+201C / U+201D (curly double quotes) and U+2018 / U+2019 (curly single
    // quotes) are a very common paste hazard and JSON.parse fails on them
    // with a generic "Unexpected token" message.
    const hasSmartQuotes = /[‘’“”]/.test(trimmed)
    try {
      const parsed = JSON.parse(trimmed)
      if (
        typeof parsed !== 'object' ||
        parsed === null ||
        Array.isArray(parsed)
      ) {
        return {
          ok: false,
          error: 'Root value must be a JSON object',
          hasSmartQuotes,
        }
      }
      return { ok: true, hasSmartQuotes: false }
    } catch (err) {
      return {
        ok: false,
        error: err instanceof Error ? err.message : String(err),
        hasSmartQuotes,
      }
    }
  }

  const isJsonValid = (value: string): boolean => validateJson(value).ok

  const extraArgsValidation = validateJson(extraArgs)
  const extraHeadersValidation = validateJson(extraHeaders)
  const extraArgsValid = extraArgsValidation.ok
  const extraHeadersValid = extraHeadersValidation.ok

  const buildExtraArgs = (): Partial<Record<string, JsonValue>> | undefined => {
    const parsed = parseJsonSafe(extraArgs) ?? {}

    // Always clear every known effort-encoding key so we never ship two forms.
    delete parsed.reasoning
    delete parsed.reasoning_effort
    delete parsed.thinking
    delete parsed.output_config

    // Registry whitelist takes priority
    const encoding = getEffortEncoding(modelId, provider, reasoningEffort)
    if (encoding) {
      Object.assign(parsed, encoding)
      if (isOpus47(modelId)) {
        delete parsed.temperature
        delete parsed.top_p
        delete parsed.top_k
      }
      return Object.keys(parsed).length > 0 ? parsed : undefined
    }

    // Fallback to old logic
    if (reasoningEffort && reasoningEffort !== 'none') {
      if (
        provider === 'anthropic' &&
        isAnthropicAdaptiveThinkingModel(modelId)
      ) {
        parsed.thinking = { type: 'adaptive' }
        parsed.output_config = { effort: reasoningEffort }
      } else if (provider === 'anthropic' && modelId.startsWith('claude-')) {
        parsed.thinking = {
          type: 'enabled',
          budget_tokens: effortToBudgetTokens(reasoningEffort),
        }
      } else if (provider === 'anthropic') {
        parsed.thinking = { type: 'enabled' }
        parsed.output_config = { effort: reasoningEffort }
      } else {
        parsed.reasoning = { effort: reasoningEffort }
      }
    } else if (
      provider === 'anthropic' &&
      !modelId.startsWith('claude-') &&
      modelId
    ) {
      parsed.thinking = { type: 'disabled' }
    }

    // Opus 4.7 rejects sampling parameters — strip them rather than 400 at runtime.
    if (isOpus47(modelId)) {
      delete parsed.temperature
      delete parsed.top_p
      delete parsed.top_k
    }

    return Object.keys(parsed).length > 0 ? parsed : undefined
  }

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
      extraArgs: buildExtraArgs(),
      extraHeaders: (() => {
        const parsed = parseJsonSafe(extraHeaders) as
          | Record<string, string>
          | null
          | undefined
        if (provider === 'anthropic' && context1MSupport) {
          return {
            ...(parsed ?? {}),
            'Anthropic-Beta': ANTHROPIC_BETA_1M_VALUE,
          }
        }
        if (
          provider === 'anthropic' &&
          !context1MSupport &&
          parsed?.['Anthropic-Beta'] === ANTHROPIC_BETA_1M_VALUE
        ) {
          const { 'Anthropic-Beta': _, ...rest } = parsed
          return Object.keys(rest).length > 0 ? rest : undefined
        }
        return parsed as Record<string, string> | null | undefined
      })(),
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
                  onChange={e => handleMaxTokensChange(e.target.value)}
                  placeholder="8192"
                  step={8192}
                />
                {isOpus47(modelId) &&
                  (reasoningEffort === 'xhigh' || reasoningEffort === 'max') &&
                  (() => {
                    const n = parseInt(maxTokens, 10)
                    return Number.isFinite(n) && n < 64000 ? (
                      <p className="text-xs text-amber-600 dark:text-amber-500">
                        {t('models.opus47.maxTokensHint')}
                      </p>
                    ) : null
                  })()}
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
                  onValueChange={handleReasoningEffortChange}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {(() => {
                      const supported = getSupportedEfforts(modelId, provider)
                      if (supported) {
                        return supported.map(effort => (
                          <SelectItem key={effort} value={effort}>
                            {t(`models.reasoningEffort.${effort}`)}
                          </SelectItem>
                        ))
                      }
                      // Fallback to old logic
                      return (
                        <>
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
                          {(provider === 'anthropic' ||
                            supportsXhighEffort(modelId)) && (
                            <SelectItem value="xhigh">
                              {t('models.reasoningEffort.xhigh')}
                            </SelectItem>
                          )}
                          {(provider === 'anthropic' ||
                            supportsMaxEffort(modelId)) && (
                            <SelectItem value="max">
                              {t('models.reasoningEffort.max')}
                            </SelectItem>
                          )}
                        </>
                      )
                    })()}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {t('models.reasoningEffortHint')}
                </p>
              </div>

              {provider === 'anthropic' && (
                <div className="flex items-center gap-2">
                  <Checkbox
                    id="context1MSupport"
                    checked={context1MSupport}
                    onCheckedChange={checked =>
                      setContext1MSupport(checked === true)
                    }
                  />
                  <Label htmlFor="context1MSupport" className="cursor-pointer">
                    {t('models.context1MSupport')}
                  </Label>
                </div>
              )}
              {provider === 'anthropic' && context1MSupport && (
                <p className="text-xs text-muted-foreground -mt-2">
                  {t('models.context1MSupportHint')}
                </p>
              )}

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
                      <div className="space-y-1">
                        <p className="text-sm text-destructive">
                          {t('models.invalidJsonWithError', {
                            error: extraArgsValidation.error,
                          })}
                        </p>
                        {extraArgsValidation.hasSmartQuotes && (
                          <p className="text-xs text-amber-600 dark:text-amber-500">
                            {t('models.smartQuotesHint')}
                          </p>
                        )}
                      </div>
                    )}
                    {isOpus47(modelId) &&
                      extraArgsValid &&
                      (() => {
                        const parsed = parseJsonSafe(extraArgs)
                        if (!parsed) return null
                        const hasForbidden =
                          'temperature' in parsed ||
                          'top_p' in parsed ||
                          'top_k' in parsed
                        return hasForbidden ? (
                          <p className="text-xs text-amber-600 dark:text-amber-500">
                            {t('models.opus47.samplingWarning')}
                          </p>
                        ) : null
                      })()}
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
                      <div className="space-y-1">
                        <p className="text-sm text-destructive">
                          {t('models.invalidJsonWithError', {
                            error: extraHeadersValidation.error,
                          })}
                        </p>
                        {extraHeadersValidation.hasSmartQuotes && (
                          <p className="text-xs text-amber-600 dark:text-amber-500">
                            {t('models.smartQuotesHint')}
                          </p>
                        )}
                      </div>
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
