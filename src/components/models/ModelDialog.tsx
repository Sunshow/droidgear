import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2 } from 'lucide-react'
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
  isOfficialModelName,
} from '@/lib/utils'

interface ModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  model?: CustomModel
  mode: 'add' | 'edit' | 'duplicate'
  onSave: (model: CustomModel) => void
}

const defaultBaseUrls: Record<Provider, string> = {
  anthropic: 'https://api.anthropic.com',
  openai: 'https://api.openai.com',
  'generic-chat-completion-api': '',
}

interface ModelFormProps {
  model?: CustomModel
  onSave: (model: CustomModel) => void
  onCancel: () => void
}

function ModelForm({ model, onSave, onCancel }: ModelFormProps) {
  const { t } = useTranslation()
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

  const handleModelIdChange = (newModelId: string) => {
    setModelId(newModelId)
    setDisplayName(newModelId)
    if (newModelId && !maxTokens) {
      setMaxTokens(getDefaultMaxOutputTokens(newModelId).toString())
    }
  }

  const handleProviderChange = (value: Provider) => {
    setProvider(value)
    // Preserve user-entered baseUrl when switching provider type.
    // Only fall back to provider defaults if baseUrl is currently empty.
    setBaseUrl(current => current || defaultBaseUrls[value])
    setAvailableModels([])
    setFetchError(null)
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
      }
    } else {
      setFetchError(result.error)
    }
  }

  const handleSave = () => {
    if (!modelId || !baseUrl || !apiKey) return

    const newModel: CustomModel = {
      model: modelId,
      baseUrl: baseUrl,
      apiKey: apiKey,
      provider,
      displayName: displayName,
      maxOutputTokens: maxTokens ? parseInt(maxTokens) : undefined,
      supportsImages: supportsImages || undefined,
    }

    onSave(newModel)
  }

  const isValid =
    modelId &&
    baseUrl &&
    apiKey &&
    displayName &&
    !containsRegexSpecialChars(displayName) &&
    !isOfficialModelName(displayName)

  return (
    <>
      <ResizableDialogBody>
        <div className="grid gap-4">
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
            {isOfficialModelName(displayName) && (
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
            />
          </div>

          <div className="flex items-center gap-2">
            <Checkbox
              id="supportsImages"
              checked={supportsImages}
              onCheckedChange={checked => setSupportsImages(checked === true)}
            />
            <Label htmlFor="supportsImages">{t('models.supportsImages')}</Label>
          </div>
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button onClick={handleSave} disabled={!isValid}>
          {model ? t('common.save') : t('common.add')}
        </Button>
      </ResizableDialogFooter>
    </>
  )
}

export function ModelDialog({
  open,
  onOpenChange,
  model,
  mode,
  onSave,
}: ModelDialogProps) {
  const { t } = useTranslation()
  const formKey = model ? `edit-${model.model}` : 'new'

  const handleSave = (newModel: CustomModel) => {
    onSave(newModel)
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
        defaultWidth={600}
        defaultHeight={580}
        minWidth={500}
        minHeight={400}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>{t(titleKey)}</ResizableDialogTitle>
        </ResizableDialogHeader>
        {open && (
          <ModelForm
            key={formKey}
            model={model}
            onSave={handleSave}
            onCancel={() => onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
