import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { FolderInput } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { SecretInput } from '@/components/ui/secret-input'
import { Label } from '@/components/ui/label'
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
import { useCodexStore } from '@/store/codex-store'
import type {
  CodexProviderConfig,
  CodexProfile,
  CustomModel,
} from '@/lib/bindings'
import { ChannelModelPickerDialog } from '@/components/channels/ChannelModelPickerDialog'
import type { ChannelProviderContext } from '@/components/channels'
import { inferModelProtocol } from '@/lib/model-protocol'
import {
  clampEffortToSupported,
  getSupportedEfforts,
} from '@/lib/model-registry'

const WIRE_API_OPTIONS = [
  { value: 'chat', label: 'Chat Completions' },
  { value: 'responses', label: 'Responses API' },
]

/** Codex-specific fallback: includes `minimal`, which is not in the shared registry type. */
const CODEX_FALLBACK_EFFORTS = [
  'none',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
] as const

const EFFORT_LABEL_KEYS: Record<string, string> = {
  none: 'models.reasoningEffort.none',
  low: 'models.reasoningEffort.low',
  medium: 'models.reasoningEffort.medium',
  high: 'models.reasoningEffort.high',
  xhigh: 'models.reasoningEffort.xhigh',
  max: 'models.reasoningEffort.max',
  minimal: 'codex.provider.reasoningEffort.minimal',
}

function effortsForModel(modelId: string): string[] {
  const trimmed = modelId.trim()
  if (!trimmed) return [...CODEX_FALLBACK_EFFORTS]
  return getSupportedEfforts(trimmed, 'openai') ?? [...CODEX_FALLBACK_EFFORTS]
}

function toStoredEffort(effort: string): string {
  return effort === 'none' ? '' : effort
}

interface ProviderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingProviderId: string | null
}

interface ProviderFormProps {
  editingProviderId: string | null
  currentProfile: CodexProfile | null
  onClose: () => void
}

function ProviderForm({
  editingProviderId,
  currentProfile,
  onClose,
}: ProviderFormProps) {
  const { t } = useTranslation()
  const addProvider = useCodexStore(state => state.addProvider)
  const updateProvider = useCodexStore(state => state.updateProvider)

  const isEditing = editingProviderId !== null
  const existingConfig = editingProviderId
    ? (
        (currentProfile?.providers ?? {}) as Record<string, CodexProviderConfig>
      )[editingProviderId]
    : null

  const [providerId, setProviderId] = useState(editingProviderId ?? '')
  const [name, setName] = useState(existingConfig?.name ?? '')
  const [baseUrl, setBaseUrl] = useState(existingConfig?.baseUrl ?? '')
  const [wireApi, setWireApi] = useState(existingConfig?.wireApi ?? 'responses')
  const [model, setModel] = useState(existingConfig?.model ?? '')
  const [modelReasoningEffort, setModelReasoningEffort] = useState(() => {
    const initialModel = existingConfig?.model ?? ''
    const options = effortsForModel(initialModel)
    const current = existingConfig?.modelReasoningEffort || 'none'
    return toStoredEffort(clampEffortToSupported(current, options))
  })
  const [apiKey, setApiKey] = useState(existingConfig?.apiKey ?? '')

  const isReservedProviderId = providerId.trim().toLowerCase() === 'openai'

  // Channel picker state
  const [channelPickerOpen, setChannelPickerOpen] = useState(false)

  const effortOptions = useMemo(() => effortsForModel(model), [model])

  // Keep Select value valid when model changes without writing during render.
  const resolvedEffort = useMemo(() => {
    const current = modelReasoningEffort || 'none'
    return toStoredEffort(clampEffortToSupported(current, effortOptions))
  }, [modelReasoningEffort, effortOptions])

  const handleModelChange = (nextModel: string) => {
    setModel(nextModel)
    const options = effortsForModel(nextModel)
    const current = modelReasoningEffort || 'none'
    setModelReasoningEffort(
      toStoredEffort(clampEffortToSupported(current, options))
    )
  }

  const sanitizeProviderId = (name: string): string => {
    return name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
  }

  const handleImportFromChannel = (
    models: CustomModel[],
    context: ChannelProviderContext
  ) => {
    if (!isEditing) {
      const sanitizedId = sanitizeProviderId(context.channelName)

      // Infer protocol from channel context
      const protocol = inferModelProtocol(
        context.channelType,
        context.platform,
        context.baseUrl
      )

      // Anthropic uses Chat Completions wire format; others use Responses API
      const inferredWireApi = protocol === 'anthropic' ? 'chat' : 'responses'

      setProviderId(sanitizedId)
      setName(context.channelName)
      setBaseUrl(context.baseUrl)
      setApiKey(context.apiKey)
      setWireApi(inferredWireApi)

      // Pre-fill model from selected model
      const selectedModel = models[0]
      if (selectedModel) {
        handleModelChange(selectedModel.model)
      }
    }
  }

  const handleSave = () => {
    if (!providerId.trim()) return

    const config: CodexProviderConfig = {
      name: name.trim() || null,
      baseUrl: baseUrl.trim() || null,
      wireApi: wireApi || null,
      requiresOpenaiAuth: null,
      envKey: null,
      envKeyInstructions: null,
      httpHeaders: null,
      queryParams: null,
      model: model.trim() || null,
      modelReasoningEffort: resolvedEffort || null,
      apiKey: apiKey.trim() || null,
    }

    if (isEditing) {
      updateProvider(providerId, config)
    } else {
      addProvider(providerId.trim(), config)
    }

    onClose()
  }

  const effortLabel = (effort: string): string => {
    const key = EFFORT_LABEL_KEYS[effort]
    if (!key) return effort
    const translated = t(key)
    // If i18n key is missing for minimal, fall back to English label.
    return translated === key && effort === 'minimal' ? 'Minimal' : translated
  }

  return (
    <>
      <ResizableDialogHeader>
        <ResizableDialogTitle>
          {isEditing ? t('codex.provider.edit') : t('codex.provider.add')}
        </ResizableDialogTitle>
        <ResizableDialogDescription>
          {t('codex.provider.dialogDescription')}
        </ResizableDialogDescription>
      </ResizableDialogHeader>

      <ResizableDialogBody>
        <div className="space-y-4">
          {/* Import from Channel */}
          {!isEditing && (
            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={() => setChannelPickerOpen(true)}
            >
              <FolderInput className="h-4 w-4 mr-2" />
              {t('codex.provider.importFromChannel')}
            </Button>
          )}

          {/* Provider ID */}
          <div className="space-y-2">
            <Label>{t('codex.provider.id')} *</Label>
            <Input
              value={providerId}
              onChange={e => setProviderId(e.target.value)}
              placeholder="custom"
              disabled={isEditing}
            />
            {isReservedProviderId && (
              <p className="text-sm text-destructive">
                {t('codex.provider.reservedProviderName')}
              </p>
            )}
          </div>

          {/* Display Name */}
          <div className="space-y-2">
            <Label>{t('codex.provider.name')}</Label>
            <Input
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="Custom Provider"
            />
          </div>

          {/* Base URL */}
          <div className="space-y-2">
            <Label>{t('codex.provider.baseUrl')}</Label>
            <Input
              value={baseUrl}
              onChange={e => setBaseUrl(e.target.value)}
              placeholder="https://api.example.com/v1"
            />
          </div>

          {/* API Key */}
          <div className="space-y-2">
            <Label>{t('codex.provider.apiKey')}</Label>
            <SecretInput
              value={apiKey}
              onChange={e => setApiKey(e.target.value)}
              placeholder="sk-..."
            />
            <p className="text-xs text-muted-foreground">
              {t('codex.provider.apiKeyHint')}
            </p>
          </div>

          {/* Wire API */}
          <div className="space-y-2">
            <Label>{t('codex.provider.wireApi')}</Label>
            <Select value={wireApi} onValueChange={setWireApi}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {WIRE_API_OPTIONS.map(option => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Model */}
          <div className="space-y-2">
            <Label>{t('codex.provider.model')}</Label>
            <Input
              value={model}
              onChange={e => handleModelChange(e.target.value)}
              placeholder={t('codex.provider.modelPlaceholder')}
            />
          </div>

          {/* Reasoning Effort */}
          <div className="space-y-2">
            <Label>{t('codex.provider.reasoningEffort')}</Label>
            <Select
              value={resolvedEffort || '__none__'}
              onValueChange={v =>
                setModelReasoningEffort(v === '__none__' ? '' : v)
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {effortOptions.map(effort => (
                  <SelectItem
                    key={effort}
                    value={effort === 'none' ? '__none__' : effort}
                  >
                    {effortLabel(effort)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button
          onClick={handleSave}
          disabled={!providerId.trim() || isReservedProviderId}
        >
          {t('common.save')}
        </Button>
      </ResizableDialogFooter>

      {/* Channel Model Picker Dialog */}
      <ChannelModelPickerDialog
        open={channelPickerOpen}
        onOpenChange={setChannelPickerOpen}
        mode="single"
        onSelect={_models => {
          // Provider-level import handled by onSelectWithContext
        }}
        onSelectWithContext={handleImportFromChannel}
        showBatchConfig={false}
        platformFilter={p => p === null || p === 'openai'}
      />
    </>
  )
}

export function ProviderDialog({
  open,
  onOpenChange,
  editingProviderId,
}: ProviderDialogProps) {
  const currentProfile = useCodexStore(state => state.currentProfile)

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={500}
        defaultHeight={520}
        minWidth={400}
        minHeight={380}
        onCloseAutoFocus={e => e.preventDefault()}
      >
        {open && (
          <ProviderForm
            editingProviderId={editingProviderId}
            currentProfile={currentProfile}
            onClose={() => onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
