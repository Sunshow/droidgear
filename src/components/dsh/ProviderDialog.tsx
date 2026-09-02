import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Download, FolderInput, LoaderCircle, Plus, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { SecretInput } from '@/components/ui/secret-input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useDshStore } from '@/store/dsh-store'
import { commands, type DshModel, type DshProviderConfig } from '@/lib/bindings'
import { findModelByIdOrAlias, getSupportedEfforts } from '@/lib/model-registry'
import { ChannelModelPickerDialog } from '@/components/channels/ChannelModelPickerDialog'
import type { ChannelProviderContext } from '@/components/channels'
import type { CustomModel } from '@/lib/bindings'

const API_OPTIONS = [
  'openai-completions',
  'openai-responses',
  'azure-openai-responses',
  'openai-codex-responses',
  'anthropic-messages',
  'google-generative-ai',
]

/** Protocols whose routes accept `compat.supportsDeveloperRole`. */
const SUPPORTS_DEV_ROLE_PROTOCOLS = new Set([
  'openai-completions',
  'openai-responses',
  'azure-openai-responses',
  'openai-codex-responses',
])

interface ProviderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingProviderId: string | null
}

interface ModelDraft {
  id: string
  name: string
  contextWindow: string
  maxTokens: string
  /** Original model, kept so reasoningEfforts/extra fields survive edits. */
  base: DshModel | null
}

function formatReasoningEfforts(
  efforts: Partial<Record<string, string | null>> | null | undefined
): string {
  if (!efforts) return ''
  return Object.entries(efforts)
    .map(([level, value]) => (value ? `${level}:${value}` : level))
    .join(', ')
}

function reasoningEffortsLabel(model: DshModel | null, id: string): string {
  const fromBase = formatReasoningEfforts(model?.reasoningEfforts)
  if (fromBase) return fromBase
  const efforts = getSupportedEfforts(id, 'openai')
  if (efforts?.length) {
    const levels = ['off', ...efforts.filter(level => level !== 'none')]
    return levels.join(', ')
  }
  const entry = findModelByIdOrAlias(id)
  return formatReasoningEfforts(entry?.thinkingLevelMap)
}

function modelToDraft(model: DshModel): ModelDraft {
  return {
    id: model.id,
    name: model.name ?? '',
    contextWindow: model.contextWindow?.toString() ?? '',
    maxTokens: model.maxTokens?.toString() ?? '',
    base: model,
  }
}

function draftToModel(draft: ModelDraft): DshModel {
  const contextWindow = draft.contextWindow.trim()
  const maxTokens = draft.maxTokens.trim()
  return {
    ...(draft.base ?? {}),
    id: draft.id.trim(),
    name: draft.name.trim() || null,
    contextWindow: contextWindow ? Number(contextWindow) : null,
    maxTokens: maxTokens ? Number(maxTokens) : null,
  }
}

function sanitizeProviderId(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function envNameForProviderId(providerId: string): string {
  return `${providerId.toUpperCase().replace(/-/g, '_')}_API_KEY`
}

function inferApiType(baseUrl: string, platform: string | null): string {
  const url = baseUrl.toLowerCase()
  if (
    platform === 'anthropic' ||
    platform === 'claude' ||
    url.includes('anthropic')
  ) {
    return 'anthropic-messages'
  }
  if (platform === 'gemini' || url.includes('googleapis')) {
    return 'google-generative-ai'
  }
  return 'openai-completions'
}

export function ProviderDialog({
  open,
  onOpenChange,
  editingProviderId,
}: ProviderDialogProps) {
  const { t } = useTranslation()
  const providers = useDshStore(state => state.providers)
  const credentials = useDshStore(state => state.credentials)
  const saveProvider = useDshStore(state => state.saveProvider)
  const saveCredential = useDshStore(state => state.saveCredential)
  const setError = useDshStore(state => state.setError)

  const isEditing = editingProviderId !== null
  const [providerId, setProviderId] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [apiKeyEnv, setApiKeyEnv] = useState('')
  const [apiKeyValue, setApiKeyValue] = useState('')
  const [api, setApi] = useState('')
  const [supportsDeveloperRole, setSupportsDeveloperRole] = useState(false)
  const [models, setModels] = useState<ModelDraft[]>([])
  const [isFetching, setIsFetching] = useState(false)
  const [validationError, setValidationError] = useState<string | null>(null)
  const [lastOpenKey, setLastOpenKey] = useState<string>('')

  // Fetch selection dialog state
  const [fetchedModels, setFetchedModels] = useState<DshModel[] | null>(null)
  const [fetchSelected, setFetchSelected] = useState<Set<string>>(new Set())
  const [fetchSelectOpen, setFetchSelectOpen] = useState(false)

  // Channel import state
  const [channelPickerOpen, setChannelPickerOpen] = useState(false)

  // Re-initialize the form whenever the dialog opens for a provider.
  const openKey = `${open}:${editingProviderId ?? ''}`
  if (open && openKey !== lastOpenKey) {
    setLastOpenKey(openKey)
    setValidationError(null)
    setIsFetching(false)
    setFetchedModels(null)
    setFetchSelectOpen(false)
    const existing = editingProviderId
      ? providers[editingProviderId]
      : undefined
    const envName = existing?.apiKeyEnv ?? ''
    setProviderId(editingProviderId ?? '')
    setDisplayName(existing?.displayName ?? '')
    setBaseUrl(existing?.baseURL ?? '')
    setApiKeyEnv(envName)
    setApiKeyValue(envName ? (credentials[envName] ?? '') : '')
    setApi(existing?.api ?? '')
    setSupportsDeveloperRole(existing?.compat?.supportsDeveloperRole ?? false)
    setModels((existing?.models ?? []).map(modelToDraft))
  }

  const handleAddModel = () => {
    setModels(prev => [
      ...prev,
      { id: '', name: '', contextWindow: '', maxTokens: '', base: null },
    ])
  }

  const handleRemoveModel = (index: number) => {
    setModels(prev => prev.filter((_, i) => i !== index))
  }

  const updateModel = (index: number, patch: Partial<ModelDraft>) => {
    setModels(prev =>
      prev.map((draft, i) => (i === index ? { ...draft, ...patch } : draft))
    )
  }

  const resolveApiKeyForFetch = (): string => {
    const direct = apiKeyValue.trim()
    if (direct) return direct
    const envName = apiKeyEnv.trim()
    return envName ? (credentials[envName] ?? '') : ''
  }

  const mergeFetchedModels = (selected: DshModel[]) => {
    setModels(prev => {
      const byId = new Map(prev.map(draft => [draft.id.trim(), draft]))
      let added = 0
      for (const model of selected) {
        const existing = byId.get(model.id)
        if (existing) {
          // Fill empty fields from the fetched entry, keep user edits.
          const merged: ModelDraft = {
            ...existing,
            name: existing.name.trim() ? existing.name : (model.name ?? ''),
            contextWindow: existing.contextWindow.trim()
              ? existing.contextWindow
              : (model.contextWindow?.toString() ?? ''),
            maxTokens: existing.maxTokens.trim()
              ? existing.maxTokens
              : (model.maxTokens?.toString() ?? ''),
            base: existing.base ?? model,
          }
          byId.set(model.id, merged)
        } else {
          byId.set(model.id, modelToDraft(model))
          added++
        }
      }
      if (added > 0) {
        toast.success(t('dsh.provider.fetchedModels', { count: added }))
      }
      return Array.from(byId.values())
    })
  }

  const handleFetchModels = async () => {
    const url = baseUrl.trim()
    const key = resolveApiKeyForFetch()
    if (!url || !key) {
      setValidationError(t('dsh.provider.fetchRequiresUrlKey'))
      return
    }
    setValidationError(null)
    setIsFetching(true)
    try {
      const result = await commands.fetchDshModels(url, key, api || null)
      if (result.status === 'ok') {
        const fetched = result.data.filter(
          model => !models.some(d => d.id.trim() === model.id)
        )
        if (fetched.length === 0) {
          toast.info(t('dsh.provider.fetchedNoNewModels'))
        } else {
          setFetchedModels(fetched)
          setFetchSelected(new Set(fetched.map(model => model.id)))
          setFetchSelectOpen(true)
        }
      } else {
        setValidationError(result.error)
      }
    } catch (e) {
      setValidationError(String(e))
    } finally {
      setIsFetching(false)
    }
  }

  const handleConfirmFetchSelection = () => {
    const pending = fetchedModels ?? []
    const selected = pending.filter(model => fetchSelected.has(model.id))
    mergeFetchedModels(selected)
    setFetchSelectOpen(false)
    setFetchedModels(null)
    setFetchSelected(new Set())
  }

  const toggleFetchSelected = (modelId: string) => {
    setFetchSelected(prev => {
      const next = new Set(prev)
      if (next.has(modelId)) {
        next.delete(modelId)
      } else {
        next.add(modelId)
      }
      return next
    })
  }

  const handleImportFromChannel = (
    selectedModels: CustomModel[],
    context: ChannelProviderContext
  ) => {
    const sanitizedId = sanitizeProviderId(context.channelName)
    setProviderId(sanitizedId)
    setDisplayName(context.channelName)
    setBaseUrl(context.baseUrl)
    const envName = envNameForProviderId(sanitizedId)
    setApiKeyEnv(envName)
    setApiKeyValue(context.apiKey)
    setApi(inferApiType(context.baseUrl, context.platform))
    setModels(
      selectedModels.map(model => ({
        id: model.model,
        name: model.displayName ?? '',
        contextWindow: '',
        maxTokens: model.maxOutputTokens?.toString() ?? '',
        base: null,
      }))
    )
  }

  const handleSave = async () => {
    const id = providerId.trim()
    if (!id) {
      setValidationError(t('dsh.provider.idRequired'))
      return
    }
    if (!isEditing && providers[id]) {
      setValidationError(t('dsh.provider.idConflict'))
      return
    }
    const modelIds = new Set<string>()
    for (const draft of models) {
      const modelId = draft.id.trim()
      if (!modelId) {
        setValidationError(t('dsh.provider.modelIdRequired'))
        return
      }
      if (modelIds.has(modelId)) {
        setValidationError(t('dsh.provider.modelIdConflict'))
        return
      }
      modelIds.add(modelId)
    }

    const base = isEditing ? providers[providerId] : undefined
    const supportsDevRole = SUPPORTS_DEV_ROLE_PROTOCOLS.has(api)
    const baseCompat = base?.compat ?? {}
    const config: DshProviderConfig = {
      ...(base ?? {}),
      displayName: displayName.trim() || null,
      baseURL: baseUrl.trim() || null,
      apiKeyEnv: apiKeyEnv.trim() || null,
      api: api || null,
      compat: supportsDevRole
        ? { ...baseCompat, supportsDeveloperRole }
        : { ...baseCompat },
      models: models.map(draftToModel),
    }

    try {
      await saveProvider(id, config)
      const envName = apiKeyEnv.trim()
      if (envName) {
        await saveCredential(envName, apiKeyValue.trim())
      }
      setError(null)
      onOpenChange(false)
    } catch {
      // store already set the error
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl sm:max-w-4xl">
        <DialogHeader>
          <DialogTitle>
            {isEditing ? t('dsh.provider.edit') : t('dsh.provider.add')}
          </DialogTitle>
          <DialogDescription>
            {t('dsh.provider.dialogDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5 py-2 max-h-[70vh] overflow-y-auto">
          {/* Import from Channel */}
          {!isEditing && (
            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={() => setChannelPickerOpen(true)}
            >
              <FolderInput className="h-4 w-4 mr-2" />
              {t('dsh.provider.importFromChannel')}
            </Button>
          )}

          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="dsh-provider-id">{t('dsh.provider.id')}</Label>
                <Input
                  id="dsh-provider-id"
                  value={providerId}
                  onChange={e => setProviderId(e.target.value)}
                  disabled={isEditing}
                  placeholder="openai"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="dsh-provider-display-name">
                  {t('dsh.provider.displayName')}
                </Label>
                <Input
                  id="dsh-provider-display-name"
                  value={displayName}
                  onChange={e => setDisplayName(e.target.value)}
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="dsh-provider-base-url">
                {t('dsh.provider.baseUrl')}
              </Label>
              <Input
                id="dsh-provider-base-url"
                value={baseUrl}
                onChange={e => setBaseUrl(e.target.value)}
                placeholder="https://api.example.com/v1"
              />
            </div>

            <div className="grid grid-cols-3 gap-4">
              <div className="space-y-2">
                <Label htmlFor="dsh-provider-api-key-env">
                  {t('dsh.provider.apiKeyEnv')}
                </Label>
                <Input
                  id="dsh-provider-api-key-env"
                  value={apiKeyEnv}
                  onChange={e => setApiKeyEnv(e.target.value)}
                  placeholder="OPENAI_API_KEY"
                />
                <p className="text-xs text-muted-foreground">
                  {t('dsh.provider.apiKeyEnvHint')}
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="dsh-provider-api-key-value">
                  {t('dsh.provider.apiKeyValue')}
                </Label>
                <SecretInput
                  id="dsh-provider-api-key-value"
                  value={apiKeyValue}
                  onChange={e => setApiKeyValue(e.target.value)}
                  disabled={!apiKeyEnv.trim()}
                  placeholder="sk-..."
                />
                <p className="text-xs text-muted-foreground">
                  {t('dsh.provider.apiKeyValueHint')}
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="dsh-provider-api">
                  {t('dsh.provider.apiType')}
                </Label>
                <Select value={api} onValueChange={setApi}>
                  <SelectTrigger id="dsh-provider-api">
                    <SelectValue
                      placeholder={t('dsh.provider.apiTypePlaceholder')}
                    />
                  </SelectTrigger>
                  <SelectContent>
                    {API_OPTIONS.map(option => (
                      <SelectItem key={option} value={option}>
                        {option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {SUPPORTS_DEV_ROLE_PROTOCOLS.has(api) && (
              <div className="flex items-center gap-2">
                <Checkbox
                  id="dsh-provider-developer-role"
                  checked={supportsDeveloperRole}
                  onCheckedChange={checked =>
                    setSupportsDeveloperRole(checked === true)
                  }
                />
                <Label
                  htmlFor="dsh-provider-developer-role"
                  className="cursor-pointer"
                >
                  {t('dsh.provider.supportsDeveloperRole')}
                </Label>
              </div>
            )}
          </div>

          <div className="space-y-2 border-t pt-4">
            <div className="flex items-center justify-between gap-2">
              <Label>{t('dsh.provider.models')}</Label>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleFetchModels}
                  disabled={isFetching || !baseUrl.trim()}
                  type="button"
                  title={t('dsh.provider.fetchModelsHint')}
                >
                  {isFetching ? (
                    <LoaderCircle
                      data-icon="inline-start"
                      className="animate-spin"
                    />
                  ) : (
                    <Download data-icon="inline-start" />
                  )}
                  {t('dsh.provider.fetchModels')}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleAddModel}
                  type="button"
                >
                  <Plus className="h-4 w-4 mr-2" />
                  {t('dsh.provider.addModel')}
                </Button>
              </div>
            </div>

            {models.length === 0 ? (
              <div className="text-center py-4 text-sm text-muted-foreground">
                {t('dsh.provider.noModels')}
              </div>
            ) : (
              <div className="space-y-2">
                {models.map((draft, index) => {
                  const effortsLabel = reasoningEffortsLabel(
                    draft.base,
                    draft.id.trim()
                  )
                  return (
                    <div
                      key={index}
                      className="border rounded-md p-3 space-y-2"
                    >
                      <div className="flex items-center gap-2">
                        <Input
                          className="flex-1 min-w-0"
                          value={draft.id}
                          onChange={e =>
                            updateModel(index, { id: e.target.value })
                          }
                          placeholder={t('dsh.provider.modelId')}
                        />
                        <Input
                          className="flex-1 min-w-0"
                          value={draft.name}
                          onChange={e =>
                            updateModel(index, { name: e.target.value })
                          }
                          placeholder={t('dsh.provider.modelName')}
                        />
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleRemoveModel(index)}
                          type="button"
                          title={t('common.delete')}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                      <div className="flex items-center gap-2">
                        <Input
                          className="w-40 shrink-0"
                          value={draft.contextWindow}
                          onChange={e =>
                            updateModel(index, {
                              contextWindow: e.target.value,
                            })
                          }
                          placeholder={t('dsh.provider.contextWindow')}
                        />
                        <Input
                          className="w-36 shrink-0"
                          value={draft.maxTokens}
                          onChange={e =>
                            updateModel(index, { maxTokens: e.target.value })
                          }
                          placeholder={t('dsh.provider.maxTokens')}
                        />
                        <span
                          className="flex-1 min-w-0 text-xs text-muted-foreground truncate"
                          title={effortsLabel}
                        >
                          {t('dsh.provider.reasoningEfforts')}:{' '}
                          {effortsLabel ||
                            t('dsh.provider.reasoningEffortsAuto')}
                        </span>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </div>

          {validationError && (
            <p className="text-sm text-destructive">{validationError}</p>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleSave}>{t('common.save')}</Button>
        </DialogFooter>
      </DialogContent>

      {/* Fetch model selection dialog */}
      <Dialog open={fetchSelectOpen} onOpenChange={setFetchSelectOpen}>
        <DialogContent className="sm:max-w-xl">
          <DialogHeader>
            <DialogTitle>{t('dsh.provider.fetchSelect.title')}</DialogTitle>
            <DialogDescription>
              {t('dsh.provider.fetchSelect.description')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2 max-h-[50vh] overflow-y-auto">
            {(fetchedModels ?? []).map(model => (
              <label
                key={model.id}
                className="flex items-center gap-3 p-2 border rounded-md hover:bg-muted/50 cursor-pointer"
              >
                <Checkbox
                  checked={fetchSelected.has(model.id)}
                  onCheckedChange={() => toggleFetchSelected(model.id)}
                />
                <span className="flex-1 min-w-0">
                  <span className="block truncate font-medium">{model.id}</span>
                  {model.name && (
                    <span className="block text-xs text-muted-foreground truncate">
                      {model.name}
                    </span>
                  )}
                </span>
              </label>
            ))}
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                const pending = fetchedModels ?? []
                setFetchSelected(new Set(pending.map(model => model.id)))
              }}
            >
              {t('dsh.provider.fetchSelect.selectAll')}
            </Button>
            <Button
              variant="outline"
              onClick={() => setFetchSelected(new Set())}
            >
              {t('dsh.provider.fetchSelect.selectNone')}
            </Button>
            <Button
              onClick={handleConfirmFetchSelection}
              disabled={fetchSelected.size === 0}
            >
              {t('dsh.provider.fetchSelect.confirm', {
                count: fetchSelected.size,
              })}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Channel Model Picker Dialog */}
      <ChannelModelPickerDialog
        open={channelPickerOpen}
        onOpenChange={setChannelPickerOpen}
        mode="multiple"
        onSelect={() => {
          // Provider-level import is handled by onSelectWithContext.
        }}
        onSelectWithContext={handleImportFromChannel}
        showBatchConfig={false}
      />
    </Dialog>
  )
}
