import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
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
import { useOpenClawStore } from '@/store/openclaw-store'
import type { OpenClawProfile, OpenClawModel } from '@/lib/bindings'

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

export function ProviderDialog({
  open,
  onOpenChange,
  editingProviderId,
  currentProfile,
}: ProviderDialogProps) {
  const { t } = useTranslation()
  const addProvider = useOpenClawStore(state => state.addProvider)
  const updateProvider = useOpenClawStore(state => state.updateProvider)

  const [providerId, setProviderId] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [api, setApi] = useState('openai-completions')
  const [models, setModels] = useState<OpenClawModel[]>([])

  const isEditing = editingProviderId !== null

  useEffect(() => {
    if (open) {
      if (editingProviderId && currentProfile) {
        const config = currentProfile.providers[editingProviderId]
        setProviderId(editingProviderId)
        setBaseUrl(config?.baseUrl ?? '')
        setApiKey(config?.apiKey ?? '')
        setApi(config?.api ?? 'openai-completions')
        setModels(config?.models ?? [])
      } else {
        setProviderId('')
        setBaseUrl('')
        setApiKey('')
        setApi('openai-completions')
        setModels([])
      }
    }
  }, [open, editingProviderId, currentProfile])

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

    onOpenChange(false)
  }

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
              <Input
                type="password"
                value={apiKey}
                onChange={e => setApiKey(e.target.value)}
                placeholder="${API_KEY}"
              />
              <p className="text-xs text-muted-foreground">
                {t('openclaw.provider.apiKeyHint')}
              </p>
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
                    <div
                      key={index}
                      className="p-3 border rounded-lg space-y-2"
                    >
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
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleSave} disabled={!providerId.trim()}>
            {t('common.save')}
          </Button>
        </ResizableDialogFooter>
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
