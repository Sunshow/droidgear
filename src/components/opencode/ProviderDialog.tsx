import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, CheckCircle, XCircle, FolderInput, Plus } from 'lucide-react'
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
import { useOpenCodeStore } from '@/store/opencode-store'
import {
  commands,
  type OpenCodeProfile,
  type OpenCodeProviderConfig,
  type OpenCodeModelConfig,
  type CustomModel,
} from '@/lib/bindings'
import { ChannelModelPickerDialog } from '@/components/channels/ChannelModelPickerDialog'
import type { ChannelProviderContext } from '@/components/channels'
import {
  inferModelProtocol,
  protocolToOpenCodeNpm,
  normalizeBaseUrlForOpenCode,
} from '@/lib/model-protocol'
import { ModelItem } from './ModelItem'
import { ModelEditDialog } from './ModelEditDialog'

interface ProviderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingProviderId: string | null
  currentProfile: OpenCodeProfile | null
}

export function ProviderDialog({
  open,
  onOpenChange,
  editingProviderId,
  currentProfile,
}: ProviderDialogProps) {
  const { t } = useTranslation()
  const providerTemplates = useOpenCodeStore(state => state.providerTemplates)
  const addProvider = useOpenCodeStore(state => state.addProvider)
  const updateProvider = useOpenCodeStore(state => state.updateProvider)

  const [providerId, setProviderId] = useState('')
  const [npm, setNpm] = useState('')
  const [name, setName] = useState('')
  const [baseURL, setBaseURL] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [timeout, setTimeout] = useState('')
  const [models, setModels] = useState<Record<string, OpenCodeModelConfig>>({})
  const [isTesting, setIsTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)
  const [channelPickerOpen, setChannelPickerOpen] = useState(false)
  const [modelEditOpen, setModelEditOpen] = useState(false)
  const [editingModelId, setEditingModelId] = useState<string | null>(null)

  const isEditing = editingProviderId !== null

  useEffect(() => {
    if (open) {
      setTestResult(null)
      if (editingProviderId && currentProfile) {
        const config = currentProfile.providers[editingProviderId]
        const auth = currentProfile.auth[editingProviderId]
        setProviderId(editingProviderId)
        setNpm(config?.npm ?? '')
        setName(config?.name ?? '')
        setBaseURL(config?.options?.baseURL ?? '')
        setTimeout(config?.options?.timeout?.toString() ?? '')
        setApiKey(
          auth && typeof auth === 'object' && 'key' in auth
            ? String(auth.key)
            : ''
        )
        // Initialize models from config
        const configModels = config?.models ?? {}
        const cleanModels: Record<string, OpenCodeModelConfig> = {}
        for (const [id, modelConfig] of Object.entries(configModels)) {
          if (modelConfig) {
            cleanModels[id] = modelConfig
          }
        }
        setModels(cleanModels)
      } else {
        setProviderId('')
        setNpm('')
        setName('')
        setBaseURL('')
        setApiKey('')
        setTimeout('')
        setModels({})
      }
    }
  }, [open, editingProviderId, currentProfile])

  const handleTemplateSelect = (templateId: string) => {
    const template = providerTemplates.find(t => t.id === templateId)
    if (template) {
      setProviderId(template.id)
      setName(template.name)
      setNpm(template.npm ?? '')
      setBaseURL(template.defaultBaseUrl ?? '')
    }
  }

  const sanitizeProviderId = (name: string): string => {
    return name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
  }

  const handleImportFromChannel = (
    selectedModels: CustomModel[],
    context: ChannelProviderContext
  ) => {
    if (!isEditing) {
      // When adding new provider, pre-fill all fields
      const sanitizedId = sanitizeProviderId(context.channelName)

      // Infer protocol from channel context (use first model ID for better inference)
      const firstModelId = selectedModels[0]?.model
      const protocol = inferModelProtocol(
        context.channelType,
        context.platform,
        context.baseUrl,
        firstModelId
      )

      // Map protocol to npm package
      const npmPackage = protocolToOpenCodeNpm(protocol)

      // Normalize baseURL for OpenCode (e.g., append /v1 for Anthropic)
      const normalizedBaseUrl = normalizeBaseUrlForOpenCode(
        protocol,
        context.baseUrl
      )

      setProviderId(sanitizedId)
      setName(context.channelName)
      setBaseURL(normalizedBaseUrl)
      setApiKey(context.apiKey)
      setNpm(npmPackage)
    }

    // Import selected models into provider config
    const importedModels: Record<string, OpenCodeModelConfig> = {}
    for (const m of selectedModels) {
      importedModels[m.model] = {
        name: m.displayName || null,
        limit: null,
      }
    }
    setModels(prev =>
      isEditing ? { ...prev, ...importedModels } : importedModels
    )
  }

  const handleTestConnection = async () => {
    if (!providerId || !baseURL || !apiKey) return
    setIsTesting(true)
    setTestResult(null)
    try {
      const result = await commands.testOpencodeProviderConnection(
        providerId,
        baseURL,
        apiKey
      )
      if (result.status === 'ok' && result.data) {
        setTestResult('success')
      } else {
        setTestResult('error')
      }
    } catch {
      setTestResult('error')
    } finally {
      setIsTesting(false)
    }
  }

  const handleSave = () => {
    if (!providerId.trim()) return

    const config: OpenCodeProviderConfig = {
      npm: npm.trim() || null,
      name: name.trim() || null,
      options: {
        baseURL: baseURL.trim() || null,
        apiKey: null,
        timeout: timeout ? parseInt(timeout, 10) : null,
        headers: null,
      },
      models: Object.keys(models).length > 0 ? models : null,
    }

    const auth = apiKey.trim() ? { type: 'api', key: apiKey.trim() } : undefined

    if (isEditing) {
      updateProvider(providerId, config, auth)
    } else {
      addProvider(providerId.trim(), config, auth)
    }

    onOpenChange(false)
  }

  const handleAddModel = () => {
    setEditingModelId(null)
    setModelEditOpen(true)
  }

  const handleEditModel = (modelId: string) => {
    setEditingModelId(modelId)
    setModelEditOpen(true)
  }

  const handleDeleteModel = (modelId: string) => {
    setModels(prev => {
      const { [modelId]: _removed, ...rest } = prev
      return rest
    })
  }

  const handleSaveModel = (modelId: string, config: OpenCodeModelConfig) => {
    setModels(prev => {
      // If editing and ID changed, remove old ID
      if (editingModelId && editingModelId !== modelId) {
        const { [editingModelId]: _removed, ...rest } = prev
        return { ...rest, [modelId]: config }
      }
      return { ...prev, [modelId]: config }
    })
  }

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={600}
        defaultHeight={580}
        minWidth={500}
        minHeight={400}
        onCloseAutoFocus={e => e.preventDefault()}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>
            {isEditing
              ? t('opencode.provider.edit')
              : t('opencode.provider.add')}
          </ResizableDialogTitle>
          <ResizableDialogDescription>
            {t('opencode.provider.dialogDescription')}
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
                {t('opencode.provider.importFromChannel')}
              </Button>
            )}

            {/* Template Selection */}
            {!isEditing && providerTemplates.length > 0 && (
              <div className="space-y-2">
                <Label>{t('opencode.provider.template')}</Label>
                <Select onValueChange={handleTemplateSelect}>
                  <SelectTrigger>
                    <SelectValue
                      placeholder={t('opencode.provider.selectTemplate')}
                    />
                  </SelectTrigger>
                  <SelectContent>
                    {providerTemplates.map(template => (
                      <SelectItem key={template.id} value={template.id}>
                        {template.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            {/* Provider ID */}
            <div className="space-y-2">
              <Label>{t('opencode.provider.id')} *</Label>
              <Input
                value={providerId}
                onChange={e => setProviderId(e.target.value)}
                placeholder="anthropic"
                disabled={isEditing}
              />
            </div>

            {/* Display Name */}
            <div className="space-y-2">
              <Label>{t('opencode.provider.displayName')}</Label>
              <Input
                value={name}
                onChange={e => setName(e.target.value)}
                placeholder="Anthropic"
              />
            </div>

            {/* NPM Package */}
            <div className="space-y-2">
              <Label>{t('opencode.provider.npm')}</Label>
              <Input
                value={npm}
                onChange={e => setNpm(e.target.value)}
                placeholder="@ai-sdk/openai-compatible"
              />
            </div>

            {/* Base URL */}
            <div className="space-y-2">
              <Label>{t('opencode.provider.baseUrl')}</Label>
              <Input
                value={baseURL}
                onChange={e => setBaseURL(e.target.value)}
                placeholder="https://api.anthropic.com"
              />
            </div>

            {/* API Key */}
            <div className="space-y-2">
              <Label>{t('opencode.provider.apiKey')}</Label>
              <Input
                type="password"
                value={apiKey}
                onChange={e => setApiKey(e.target.value)}
                placeholder="sk-ant-..."
              />
            </div>

            {/* Timeout */}
            <div className="space-y-2">
              <Label>{t('opencode.provider.timeout')}</Label>
              <Input
                type="number"
                value={timeout}
                onChange={e => setTimeout(e.target.value)}
                placeholder="300000"
              />
            </div>

            {/* Models Section */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>{t('opencode.provider.models')}</Label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={handleAddModel}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  {t('opencode.provider.addModel')}
                </Button>
              </div>

              {Object.keys(models).length === 0 ? (
                <div className="text-sm text-muted-foreground text-center py-4 border rounded-md">
                  {t('opencode.provider.noModels')}
                </div>
              ) : (
                <div className="space-y-2 border rounded-md p-2 max-h-[200px] overflow-y-auto">
                  {Object.entries(models).map(([modelId, modelConfig]) => (
                    <ModelItem
                      key={modelId}
                      modelId={modelId}
                      config={modelConfig}
                      onEdit={() => handleEditModel(modelId)}
                      onDelete={() => handleDeleteModel(modelId)}
                    />
                  ))}
                </div>
              )}
            </div>

            {/* Test Connection */}
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleTestConnection}
                disabled={!providerId || !baseURL || !apiKey || isTesting}
              >
                {isTesting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                {t('opencode.provider.testConnection')}
              </Button>
              {testResult === 'success' && (
                <span className="flex items-center text-sm text-green-600">
                  <CheckCircle className="h-4 w-4 mr-1" />
                  {t('opencode.provider.testSuccess')}
                </span>
              )}
              {testResult === 'error' && (
                <span className="flex items-center text-sm text-destructive">
                  <XCircle className="h-4 w-4 mr-1" />
                  {t('opencode.provider.testFailed')}
                </span>
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

      {/* Channel Model Picker Dialog */}
      <ChannelModelPickerDialog
        open={channelPickerOpen}
        onOpenChange={setChannelPickerOpen}
        mode="multiple"
        onSelect={_models => {
          // Provider-level import handled by onSelectWithContext
        }}
        onSelectWithContext={handleImportFromChannel}
        showBatchConfig={false}
      />

      {/* Model Edit Dialog */}
      <ModelEditDialog
        key={editingModelId || 'new'}
        open={modelEditOpen}
        onOpenChange={setModelEditOpen}
        modelId={editingModelId}
        config={editingModelId ? (models[editingModelId] ?? null) : null}
        existingModelIds={Object.keys(models)}
        onSave={handleSaveModel}
      />
    </ResizableDialog>
  )
}
