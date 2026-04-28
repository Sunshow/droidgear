import { useState, useCallback, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { SecretInput } from '@/components/ui/secret-input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { Textarea } from '@/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { usePiStore } from '@/store/pi-store'
import type {
  PiProfile,
  PiProviderConfig,
  PiModel,
  PiCompatConfig,
  PiModelCost,
} from '@/lib/bindings'

const API_TYPES = [
  { value: 'openai-completions', label: 'OpenAI Completions' },
  { value: 'openai-responses', label: 'OpenAI Responses' },
  { value: 'anthropic-messages', label: 'Anthropic Messages' },
  { value: 'google-generative-ai', label: 'Google Generative AI' },
]

interface ProviderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingProviderId: string | null
  currentProfile: PiProfile | null
}

interface JsonFieldProps {
  label: string
  value: string
  onChange: (value: string) => void
  error: string | null
  placeholder?: string
}

function JsonField({
  label,
  value,
  onChange,
  error,
  placeholder,
}: JsonFieldProps) {
  return (
    <div className="space-y-2">
      <Label className="text-xs">{label}</Label>
      <Textarea
        value={value}
        onChange={e => onChange(e.target.value)}
        placeholder={placeholder ?? '{}'}
        className={`font-mono text-xs min-h-[60px] ${error ? 'border-destructive' : ''}`}
      />
      {error && <p className="text-xs text-destructive">{error}</p>}
    </div>
  )
}

function validateJson(str: string): { valid: boolean; error: string | null } {
  if (!str.trim()) return { valid: true, error: null }
  try {
    JSON.parse(str)
    return { valid: true, error: null }
  } catch (e) {
    return {
      valid: false,
      error: `Invalid JSON: ${e instanceof Error ? e.message : 'parse error'}`,
    }
  }
}

interface ProviderFormProps {
  editingProviderId: string | null
  currentProfile: PiProfile | null
  onClose: () => void
}

function ProviderForm({
  editingProviderId,
  currentProfile,
  onClose,
}: ProviderFormProps) {
  const { t } = useTranslation()
  const addProvider = usePiStore(state => state.addProvider)
  const updateProvider = usePiStore(state => state.updateProvider)

  const isEditing = editingProviderId !== null
  const existingConfig = editingProviderId
    ? (currentProfile?.providers?.[editingProviderId] ?? undefined)
    : undefined

  const [providerId, setProviderId] = useState(editingProviderId ?? '')
  const [baseUrl, setBaseUrl] = useState(existingConfig?.baseUrl ?? '')
  const [apiKey, setApiKey] = useState(existingConfig?.apiKey ?? '')
  const [api, setApi] = useState(existingConfig?.api ?? 'openai-completions')
  const [authHeader, setAuthHeader] = useState(
    existingConfig?.authHeader ?? false
  )
  const [models, setModels] = useState<PiModel[]>(existingConfig?.models ?? [])

  // JSON fields
  const [headersStr, setHeadersStr] = useState(
    existingConfig?.headers
      ? JSON.stringify(existingConfig.headers, null, 2)
      : ''
  )
  const [compatStr, setCompatStr] = useState(
    existingConfig?.compat ? JSON.stringify(existingConfig.compat, null, 2) : ''
  )
  const [modelOverridesStr, setModelOverridesStr] = useState(
    existingConfig?.modelOverrides
      ? JSON.stringify(existingConfig.modelOverrides, null, 2)
      : ''
  )

  // JSON validation errors
  const [headersError, setHeadersError] = useState<string | null>(null)
  const [compatError, setCompatError] = useState<string | null>(null)
  const [modelOverridesError, setModelOverridesError] = useState<string | null>(
    null
  )

  // Track model-level compat/json errors
  const [modelCompatErrors, setModelCompatErrors] = useState<
    Record<number, string | null>
  >({})
  const [modelCostErrors, setModelCostErrors] = useState<
    Record<number, string | null>
  >({})

  const modelsContainerRef = useRef<HTMLDivElement>(null)

  const validateHeaders = useCallback((val: string) => {
    const result = validateJson(val)
    setHeadersError(result.error)
    return result.valid
  }, [])

  const validateCompat = useCallback((val: string) => {
    const result = validateJson(val)
    setCompatError(result.error)
    return result.valid
  }, [])

  const validateModelOverrides = useCallback((val: string) => {
    const result = validateJson(val)
    setModelOverridesError(result.error)
    return result.valid
  }, [])

  const handleAddModel = () => {
    const newModel: PiModel = {
      id: '',
      name: null,
      api: null,
      reasoning: false,
      input: ['text'],
      contextWindow: undefined,
      maxTokens: undefined,
      cost: null,
      compat: null,
    }
    setModels([...models, newModel])
    setTimeout(() => {
      modelsContainerRef.current?.scrollTo({
        top: modelsContainerRef.current.scrollHeight,
        behavior: 'smooth',
      })
    }, 0)
  }

  const handleRemoveModel = (index: number) => {
    setModels(models.filter((_, i) => i !== index))
    // Clean up error state
    setModelCompatErrors(
      prev =>
        Object.fromEntries(
          Object.entries(prev).filter(([k]) => k !== String(index))
        ) as Record<number, string | null>
    )
    setModelCostErrors(
      prev =>
        Object.fromEntries(
          Object.entries(prev).filter(([k]) => k !== String(index))
        ) as Record<number, string | null>
    )
  }

  const handleModelChange = (
    index: number,
    field: keyof PiModel,
    value: string | boolean | string[] | number | undefined | null
  ) => {
    const updated = [...models]
    const model = updated[index]
    if (!model) return
    updated[index] = { ...model, [field]: value }
    setModels(updated)
  }

  const handleModelCompatChange = (index: number, value: string) => {
    const result = validateJson(value)
    setModelCompatErrors(prev => ({ ...prev, [index]: result.error }))
    if (result.valid && value.trim()) {
      try {
        const parsed = JSON.parse(value) as PiCompatConfig
        const updated = [...models]
        const model = updated[index]
        if (!model) return
        updated[index] = { ...model, compat: parsed }
        setModels(updated)
      } catch {
        // ignore
      }
    } else if (!value.trim()) {
      const updated = [...models]
      const model = updated[index]
      if (!model) return
      updated[index] = { ...model, compat: null }
      setModels(updated)
    }
  }

  const handleModelCostChange = (
    index: number,
    field: keyof PiModelCost,
    value: string
  ) => {
    const updated = [...models]
    const model = updated[index]
    if (!model) return
    const currentCost = model.cost ?? {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
    }
    const numValue = value === '' ? undefined : parseFloat(value)
    const newCost: PiModelCost = {
      ...currentCost,
      [field]: numValue,
    }
    updated[index] = { ...model, cost: newCost }
    setModels(updated)
  }

  const hasValidationErrors = () => {
    if (headersError) return true
    if (compatError) return true
    if (modelOverridesError) return true
    if (Object.values(modelCompatErrors).some(e => e !== null)) return true
    if (Object.values(modelCostErrors).some(e => e !== null)) return true
    return false
  }

  const handleSave = () => {
    if (!providerId.trim()) return
    if (hasValidationErrors()) {
      toast.error(t('pi.provider.validationError'))
      return
    }

    // Parse JSON fields
    let headers: Record<string, string> | null = null
    if (headersStr.trim()) {
      try {
        headers = JSON.parse(headersStr)
      } catch {
        toast.error(t('pi.provider.headersInvalid'))
        return
      }
    }

    let compat: PiCompatConfig | null = null
    if (compatStr.trim()) {
      try {
        compat = JSON.parse(compatStr)
      } catch {
        toast.error(t('pi.provider.compatInvalid'))
        return
      }
    }

    let modelOverrides: Record<string, unknown> | null = null
    if (modelOverridesStr.trim()) {
      try {
        modelOverrides = JSON.parse(modelOverridesStr)
      } catch {
        toast.error(t('pi.provider.modelOverridesInvalid'))
        return
      }
    }

    // Filter out models with empty IDs
    const validModels = models.filter(m => m.id.trim())

    const config: PiProviderConfig = {
      baseUrl: baseUrl.trim() || null,
      api: api || null,
      apiKey: apiKey.trim() || null,
      headers: headers as PiProviderConfig['headers'],
      authHeader: authHeader || null,
      models: validModels,
      modelOverrides: modelOverrides as PiProviderConfig['modelOverrides'],
      compat,
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
      <div
        className="flex-1 overflow-y-auto px-6 py-4 space-y-4"
        ref={modelsContainerRef}
      >
        {/* Provider ID */}
        <div className="space-y-2">
          <Label>{t('pi.provider.id')} *</Label>
          <Input
            value={providerId}
            onChange={e => setProviderId(e.target.value)}
            placeholder="custom-provider"
            disabled={isEditing}
          />
        </div>

        {/* Base URL */}
        <div className="space-y-2">
          <Label>{t('pi.provider.baseUrl')}</Label>
          <Input
            value={baseUrl}
            onChange={e => setBaseUrl(e.target.value)}
            placeholder="https://api.example.com/v1"
          />
        </div>

        {/* API Type */}
        <div className="space-y-2">
          <Label>{t('pi.provider.apiType')}</Label>
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

        {/* API Key */}
        <div className="space-y-2">
          <Label>{t('pi.provider.apiKey')}</Label>
          <SecretInput
            value={apiKey}
            onChange={e => setApiKey(e.target.value)}
            placeholder="${API_KEY}"
          />
          <p className="text-xs text-muted-foreground">
            {t('pi.provider.apiKeyHint')}
          </p>
        </div>

        {/* Headers JSON */}
        <JsonField
          label={t('pi.provider.headers')}
          value={headersStr}
          onChange={val => {
            setHeadersStr(val)
            validateHeaders(val)
          }}
          error={headersError}
          placeholder='{"X-Custom-Header": "value"}'
        />

        {/* Auth Header Checkbox */}
        <div className="flex items-center gap-2">
          <Checkbox
            id="authHeader"
            checked={authHeader}
            onCheckedChange={checked => setAuthHeader(!!checked)}
          />
          <Label htmlFor="authHeader" className="cursor-pointer">
            {t('pi.provider.authHeader')}
          </Label>
        </div>

        {/* Compat JSON */}
        <JsonField
          label={t('pi.provider.compat')}
          value={compatStr}
          onChange={val => {
            setCompatStr(val)
            validateCompat(val)
          }}
          error={compatError}
          placeholder='{"supportsDeveloperRole": true}'
        />

        {/* Model Overrides JSON */}
        <JsonField
          label={t('pi.provider.modelOverrides')}
          value={modelOverridesStr}
          onChange={val => {
            setModelOverridesStr(val)
            validateModelOverrides(val)
          }}
          error={modelOverridesError}
        />

        {/* Models Section */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label className="text-base font-medium">
              {t('pi.provider.models')}
            </Label>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleAddModel}
            >
              <Plus className="h-4 w-4 mr-1" />
              {t('pi.provider.addModel')}
            </Button>
          </div>

          {models.length === 0 ? (
            <p className="text-sm text-muted-foreground py-2">
              {t('pi.provider.noModels')}
            </p>
          ) : (
            <div className="space-y-3">
              {models.map((model, index) => (
                <div key={index} className="p-3 border rounded-lg space-y-2">
                  <div className="flex items-center justify-between">
                    <Label className="text-sm font-medium">
                      {t('pi.provider.model')} #{index + 1}
                      {model.id ? ` — ${model.id}` : ''}
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
                    {/* Model ID */}
                    <div>
                      <Label className="text-xs">
                        {t('pi.provider.modelId')} *
                      </Label>
                      <Input
                        value={model.id}
                        onChange={e =>
                          handleModelChange(index, 'id', e.target.value)
                        }
                        placeholder="llama3.1:8b"
                        className="h-8"
                      />
                    </div>
                    {/* Model Name */}
                    <div>
                      <Label className="text-xs">
                        {t('pi.provider.modelName')}
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
                        placeholder="Llama 3.1 8B"
                        className="h-8"
                      />
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-2">
                    {/* Context Window */}
                    <div>
                      <Label className="text-xs">
                        {t('pi.provider.contextWindow')}
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
                              : undefined
                          )
                        }
                        placeholder="128000"
                        className="h-8"
                      />
                    </div>
                    {/* Max Tokens */}
                    <div>
                      <Label className="text-xs">
                        {t('pi.provider.maxTokens')}
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
                              : undefined
                          )
                        }
                        placeholder="16384"
                        className="h-8"
                      />
                    </div>
                  </div>

                  {/* Reasoning Checkbox */}
                  <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id={`reasoning-${index}`}
                        checked={model.reasoning ?? false}
                        onCheckedChange={checked =>
                          handleModelChange(index, 'reasoning', !!checked)
                        }
                      />
                      <Label
                        htmlFor={`reasoning-${index}`}
                        className="text-xs cursor-pointer"
                      >
                        {t('pi.provider.reasoning')}
                      </Label>
                    </div>
                    {/* Input types */}
                    <div className="flex items-center gap-2">
                      <Label className="text-xs">
                        {t('pi.provider.inputTypes')}:
                      </Label>
                      <div className="flex items-center gap-1">
                        <Checkbox
                          id={`input-text-${index}`}
                          checked={(model.input ?? []).includes('text')}
                          onCheckedChange={checked => {
                            const current = model.input ?? []
                            const next = checked
                              ? [...new Set([...current, 'text'])]
                              : current.filter(i => i !== 'text')
                            handleModelChange(index, 'input', next)
                          }}
                        />
                        <Label
                          htmlFor={`input-text-${index}`}
                          className="text-xs cursor-pointer"
                        >
                          {t('pi.provider.inputText')}
                        </Label>
                      </div>
                      <div className="flex items-center gap-1">
                        <Checkbox
                          id={`input-image-${index}`}
                          checked={(model.input ?? []).includes('image')}
                          onCheckedChange={checked => {
                            const current = model.input ?? []
                            const next = checked
                              ? [...new Set([...current, 'image'])]
                              : current.filter(i => i !== 'image')
                            handleModelChange(index, 'input', next)
                          }}
                        />
                        <Label
                          htmlFor={`input-image-${index}`}
                          className="text-xs cursor-pointer"
                        >
                          {t('pi.provider.inputImage')}
                        </Label>
                      </div>
                    </div>
                  </div>

                  {/* Cost Fields */}
                  <div className="space-y-1">
                    <Label className="text-xs">{t('pi.provider.cost')}</Label>
                    <div className="grid grid-cols-4 gap-2">
                      <div>
                        <Label className="text-[10px] text-muted-foreground">
                          {t('pi.provider.costInput')}
                        </Label>
                        <Input
                          type="number"
                          step="0.000001"
                          value={model.cost?.input ?? ''}
                          onChange={e =>
                            handleModelCostChange(
                              index,
                              'input',
                              e.target.value
                            )
                          }
                          placeholder="0"
                          className="h-7 text-xs"
                        />
                      </div>
                      <div>
                        <Label className="text-[10px] text-muted-foreground">
                          {t('pi.provider.costOutput')}
                        </Label>
                        <Input
                          type="number"
                          step="0.000001"
                          value={model.cost?.output ?? ''}
                          onChange={e =>
                            handleModelCostChange(
                              index,
                              'output',
                              e.target.value
                            )
                          }
                          placeholder="0"
                          className="h-7 text-xs"
                        />
                      </div>
                      <div>
                        <Label className="text-[10px] text-muted-foreground">
                          {t('pi.provider.costCacheRead')}
                        </Label>
                        <Input
                          type="number"
                          step="0.000001"
                          value={model.cost?.cacheRead ?? ''}
                          onChange={e =>
                            handleModelCostChange(
                              index,
                              'cacheRead',
                              e.target.value
                            )
                          }
                          placeholder="0"
                          className="h-7 text-xs"
                        />
                      </div>
                      <div>
                        <Label className="text-[10px] text-muted-foreground">
                          {t('pi.provider.costCacheWrite')}
                        </Label>
                        <Input
                          type="number"
                          step="0.000001"
                          value={model.cost?.cacheWrite ?? ''}
                          onChange={e =>
                            handleModelCostChange(
                              index,
                              'cacheWrite',
                              e.target.value
                            )
                          }
                          placeholder="0"
                          className="h-7 text-xs"
                        />
                      </div>
                    </div>
                  </div>

                  {/* Model Compat JSON */}
                  <JsonField
                    label={t('pi.provider.modelCompat')}
                    value={
                      model.compat ? JSON.stringify(model.compat, null, 2) : ''
                    }
                    onChange={val => handleModelCompatChange(index, val)}
                    error={modelCompatErrors[index] ?? null}
                    placeholder='{"supportsDeveloperRole": true}'
                  />
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="flex justify-end gap-2 px-6 py-4 border-t">
        <Button variant="outline" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button
          onClick={handleSave}
          disabled={!providerId.trim() || hasValidationErrors()}
        >
          {t('common.save')}
        </Button>
      </div>
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
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>
            {isEditing ? t('pi.provider.edit') : t('pi.provider.add')}
          </DialogTitle>
          <DialogDescription>
            {t('pi.provider.dialogDescription')}
          </DialogDescription>
        </DialogHeader>
        {open && (
          <ProviderForm
            key={formKey}
            editingProviderId={editingProviderId}
            currentProfile={currentProfile}
            onClose={() => onOpenChange(false)}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}
