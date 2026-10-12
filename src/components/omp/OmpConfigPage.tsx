import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  Play,
  Plus,
  Copy,
  Trash2,
  Download,
  RefreshCw,
  CloudDownload,
  X,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { SecretInput } from '@/components/ui/secret-input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
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
import { useOmpStore } from '@/store/omp-store'
import { ConfigStatus } from './ConfigStatus'
import { ProviderCard } from './ProviderCard'
import { TooltipProvider } from '@/components/ui/tooltip'
import { ChannelModelPickerDialog } from '@/components/channels/ChannelModelPickerDialog'
import type {
  OmpCustomModel,
  OmpCustomProvider,
  OmpCustomProviderEntry,
  OmpProfile,
} from '@/lib/bindings'

const ROLE_KEYS = ['default', 'smol', 'slow', 'plan', 'commit'] as const

function ProfileFields({
  profile,
  onSave,
  t,
}: {
  profile: OmpProfile
  onSave: (patch: Partial<OmpProfile>) => Promise<void>
  t: (key: string) => string
}) {
  const [name, setName] = useState(profile.name)
  const [description, setDescription] = useState(profile.description ?? '')

  const handleBlur = async () => {
    if (name !== profile.name || description !== (profile.description ?? '')) {
      await onSave({
        name,
        description: description || undefined,
      })
    }
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <Label className="w-24 shrink-0">{t('omp.profile.name')}</Label>
        <Input
          className="flex-1"
          value={name}
          onChange={e => setName(e.target.value)}
          onBlur={handleBlur}
          placeholder={t('omp.profile.namePlaceholder')}
        />
      </div>
      <div className="flex items-center gap-2">
        <Label className="w-24 shrink-0">{t('omp.profile.description')}</Label>
        <Input
          className="flex-1"
          value={description}
          onChange={e => setDescription(e.target.value)}
          onBlur={handleBlur}
          placeholder={t('omp.profile.descriptionPlaceholder')}
        />
      </div>
    </div>
  )
}

function OmpProviderEditor({
  open,
  onOpenChange,
  entry,
  onSave,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  entry: OmpCustomProviderEntry | null
  onSave: (providerId: string, provider: OmpCustomProvider) => Promise<void>
}) {
  const { t } = useTranslation()
  const editing = entry !== null
  const [providerId, setProviderId] = useState(entry?.providerId ?? '')
  const [baseUrl, setBaseUrl] = useState(entry?.config.baseUrl ?? '')
  const [api, setApi] = useState(entry?.config.api ?? 'openai-completions')
  const [apiKey, setApiKey] = useState(entry?.config.apiKey ?? '')
  const [auth, setAuth] = useState(entry?.config.auth ?? 'apiKey')
  const [models, setModels] = useState<OmpCustomModel[]>(
    entry?.config.models ?? []
  )
  const [pickerOpen, setPickerOpen] = useState(false)

  const updateModel = (index: number, patch: Partial<OmpCustomModel>) => {
    setModels(current =>
      current.map((model, modelIndex) =>
        modelIndex === index ? { ...model, ...patch } : model
      )
    )
  }

  const addModel = () => setModels(current => [...current, { id: '' }])

  const importedModels = (
    selected: {
      model: string
      baseUrl: string
      apiKey: string
      provider: string
    }[],
    context?: { baseUrl: string; apiKey: string; provider?: string }
  ) => {
    const importedApi =
      context?.provider === 'anthropic'
        ? 'anthropic-messages'
        : context?.provider === 'openai'
          ? 'openai-completions'
          : undefined
    if (context) {
      setBaseUrl(context.baseUrl)
      setApiKey(context.apiKey)
      setAuth('apiKey')
      if (importedApi) setApi(importedApi)
    }
    setModels(current => {
      const byId = new Map(current.map(model => [model.id, model]))
      for (const model of selected) {
        const previous = byId.get(model.model)
        byId.set(model.model, {
          ...previous,
          id: model.model,
          name: previous?.name ?? model.model,
          api: importedApi ?? previous?.api ?? api,
          baseUrl: model.baseUrl || previous?.baseUrl,
          reasoning: previous?.reasoning ?? false,
          input: previous?.input ?? ['text'],
        })
      }
      return Array.from(byId.values())
    })
  }

  const handleSave = async () => {
    const id = providerId.trim()
    if (!id) return
    await onSave(id, {
      ...(entry?.config ?? {}),
      baseUrl: baseUrl.trim() || undefined,
      api: api || undefined,
      apiKey: apiKey.trim() || undefined,
      auth: auth || undefined,
      models: models.filter(model => model.id.trim()),
    })
    onOpenChange(false)
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {editing ? t('omp.provider.edit') : t('omp.provider.add')}
            </DialogTitle>
            <DialogDescription>
              {t('omp.provider.description')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-2">
                <Label>{t('omp.provider.id')} *</Label>
                <Input
                  value={providerId}
                  onChange={e => setProviderId(e.target.value)}
                  disabled={editing}
                  placeholder="custom-provider"
                />
              </div>
              <div className="space-y-2">
                <Label>{t('omp.provider.baseUrl')}</Label>
                <Input
                  value={baseUrl}
                  onChange={e => setBaseUrl(e.target.value)}
                  placeholder="https://api.example.com/v1"
                />
              </div>
            </div>
            <div className="grid grid-cols-3 gap-3">
              <div className="space-y-2">
                <Label>{t('omp.provider.api')}</Label>
                <Select value={api} onValueChange={setApi}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="openai-completions">
                      OpenAI Completions
                    </SelectItem>
                    <SelectItem value="openai-responses">
                      OpenAI Responses
                    </SelectItem>
                    <SelectItem value="anthropic-messages">
                      Anthropic Messages
                    </SelectItem>
                    <SelectItem value="google-generative-ai">
                      Google Generative AI
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>{t('omp.provider.apiKey')}</Label>
                <SecretInput
                  value={apiKey}
                  onChange={e => setApiKey(e.target.value)}
                  placeholder="${API_KEY}"
                />
              </div>
              <div className="space-y-2">
                <Label>{t('omp.provider.auth')}</Label>
                <Select value={auth} onValueChange={setAuth}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="apiKey">API key</SelectItem>
                    <SelectItem value="none">None</SelectItem>
                    <SelectItem value="oauth">OAuth</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>{t('omp.provider.models')}</Label>
                <div className="flex gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => setPickerOpen(true)}
                  >
                    <CloudDownload className="h-4 w-4 mr-1" />
                    {t('omp.provider.importFromChannel')}
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={addModel}
                  >
                    <Plus className="h-4 w-4 mr-1" />
                    {t('omp.provider.addModel')}
                  </Button>
                </div>
              </div>
              {models.length === 0 && (
                <p className="text-sm text-muted-foreground">
                  {t('omp.provider.noModels')}
                </p>
              )}
              <div className="space-y-2">
                {models.map((model, index) => (
                  <div
                    key={`${model.id}-${index}`}
                    className="grid grid-cols-[1fr_1fr_1fr_auto] gap-2 items-end border rounded p-2"
                  >
                    <div className="space-y-1">
                      <Label className="text-xs">
                        {t('omp.provider.modelId')}
                      </Label>
                      <Input
                        value={model.id}
                        onChange={e =>
                          updateModel(index, { id: e.target.value })
                        }
                      />
                    </div>
                    <div className="space-y-1">
                      <Label className="text-xs">
                        {t('omp.provider.modelName')}
                      </Label>
                      <Input
                        value={model.name ?? ''}
                        onChange={e =>
                          updateModel(index, {
                            name: e.target.value || undefined,
                          })
                        }
                      />
                    </div>
                    <div className="space-y-1">
                      <Label className="text-xs">
                        {t('omp.provider.modelBaseUrl')}
                      </Label>
                      <Input
                        value={model.baseUrl ?? ''}
                        onChange={e =>
                          updateModel(index, {
                            baseUrl: e.target.value || undefined,
                          })
                        }
                      />
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      onClick={() =>
                        setModels(current =>
                          current.filter(
                            (_, modelIndex) => modelIndex !== index
                          )
                        )
                      }
                      aria-label={t('common.delete')}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                    <div className="col-span-3 flex items-center gap-4">
                      <div className="flex items-center gap-2">
                        <Checkbox
                          checked={model.reasoning ?? false}
                          onCheckedChange={checked =>
                            updateModel(index, { reasoning: !!checked })
                          }
                        />
                        <Label className="text-xs">
                          {t('omp.provider.reasoning')}
                        </Label>
                      </div>
                      <div className="flex items-center gap-2">
                        <Label className="text-xs">
                          {t('omp.provider.contextWindow')}
                        </Label>
                        <Input
                          className="w-28 h-8"
                          type="number"
                          value={model.contextWindow ?? ''}
                          onChange={e =>
                            updateModel(index, {
                              contextWindow: e.target.value
                                ? Number(e.target.value)
                                : undefined,
                            })
                          }
                        />
                      </div>
                      <div className="flex items-center gap-2">
                        <Label className="text-xs">
                          {t('omp.provider.maxTokens')}
                        </Label>
                        <Input
                          className="w-28 h-8"
                          type="number"
                          value={model.maxTokens ?? ''}
                          onChange={e =>
                            updateModel(index, {
                              maxTokens: e.target.value
                                ? Number(e.target.value)
                                : undefined,
                            })
                          }
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={handleSave} disabled={!providerId.trim()}>
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ChannelModelPickerDialog
        open={pickerOpen}
        onOpenChange={setPickerOpen}
        mode="multiple"
        existingModels={models.map(model => ({
          model: model.id,
          baseUrl,
          apiKey,
          provider: 'generic-chat-completion-api',
        }))}
        onSelect={selected => importedModels(selected)}
        onSelectWithContext={(selected, context) =>
          importedModels(selected, context)
        }
      />
    </>
  )
}

export function OmpConfigPage() {
  const { t } = useTranslation()
  const profiles = useOmpStore(state => state.profiles)
  const activeProfileId = useOmpStore(state => state.activeProfileId)
  const currentProfile = useOmpStore(state => state.currentProfile)
  const configStatus = useOmpStore(state => state.configStatus)
  const liveConfig = useOmpStore(state => state.liveConfig)
  const customProviders = useOmpStore(state => state.customProviders)
  const loadProfiles = useOmpStore(state => state.loadProfiles)
  const loadActiveProfileId = useOmpStore(state => state.loadActiveProfileId)
  const loadConfigStatus = useOmpStore(state => state.loadConfigStatus)
  const loadLiveConfig = useOmpStore(state => state.loadLiveConfig)
  const loadCustomProviders = useOmpStore(state => state.loadCustomProviders)
  const saveCustomProvider = useOmpStore(state => state.saveCustomProvider)
  const deleteCustomProvider = useOmpStore(state => state.deleteCustomProvider)
  const selectProfile = useOmpStore(state => state.selectProfile)
  const createProfile = useOmpStore(state => state.createProfile)
  const saveProfile = useOmpStore(state => state.saveProfile)
  const deleteProfile = useOmpStore(state => state.deleteProfile)
  const duplicateProfile = useOmpStore(state => state.duplicateProfile)
  const applyProfile = useOmpStore(state => state.applyProfile)

  const [providerEditorOpen, setProviderEditorOpen] = useState(false)
  const [editingProvider, setEditingProvider] =
    useState<OmpCustomProviderEntry | null>(null)
  const [providerToDelete, setProviderToDelete] = useState<string | null>(null)

  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [newProfileName, setNewProfileName] = useState('')

  useEffect(() => {
    void loadProfiles().then(() => {
      void loadActiveProfileId()
    })
    void loadConfigStatus()
    void loadLiveConfig()
    void loadCustomProviders()
  }, [
    loadProfiles,
    loadActiveProfileId,
    loadConfigStatus,
    loadLiveConfig,
    loadCustomProviders,
  ])

  const openNewProvider = () => {
    setEditingProvider(null)
    setProviderEditorOpen(true)
  }

  const openEditProvider = (entry: OmpCustomProviderEntry) => {
    setEditingProvider(entry)
    setProviderEditorOpen(true)
  }

  const handleDeleteProvider = async () => {
    if (!providerToDelete) return
    await deleteCustomProvider(providerToDelete)
    setProviderToDelete(null)
  }

  const handleSaveProvider = async (
    providerId: string,
    provider: OmpCustomProvider
  ) => {
    await saveCustomProvider(providerId, provider)
  }

  const handleProfileChange = (id: string) => {
    selectProfile(id)
  }

  const handleCreateProfile = async () => {
    if (!newProfileName.trim()) return
    await createProfile(newProfileName.trim())
    setShowCreateDialog(false)
    setNewProfileName('')
  }

  const handleApply = async () => {
    if (!currentProfile) return
    await applyProfile(currentProfile.id)
    const currentError = useOmpStore.getState().error
    if (!currentError) {
      toast.success(t('omp.actions.applySuccess'))
    }
  }

  const updateCurrentProfile = async (patch: Partial<OmpProfile>) => {
    const store = useOmpStore.getState()
    if (!store.currentProfile) return
    const updated = { ...store.currentProfile, ...patch }
    store.selectProfile(store.currentProfile.id)
    useOmpStore.setState({ currentProfile: updated })
    await saveProfile()
  }

  const handleLoadFromLive = async () => {
    if (!currentProfile) return
    // Reload live config first in case it changed
    await loadLiveConfig()
    const latest = useOmpStore.getState().liveConfig
    if (latest?.agentConfig?.modelRoles) {
      await updateCurrentProfile({
        modelRoles: latest.agentConfig.modelRoles,
      })
      toast.success(t('omp.actions.loadedFromLive'))
    } else {
      toast.warning(t('omp.actions.noLiveRoles'))
    }
  }

  const handleModelRoleChange = async (role: string, value: string) => {
    if (!currentProfile) return
    await updateCurrentProfile({
      modelRoles: {
        ...currentProfile.modelRoles,
        [role]: value || undefined,
      },
    })
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('omp.title')}</h1>
          <div className="flex items-center gap-2 mt-1">
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('omp.profile.active')}</Badge>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => loadLiveConfig()}
            title={t('omp.actions.refreshLive')}
          >
            <RefreshCw className="h-4 w-4 mr-1" />
            {t('omp.actions.refreshLive')}
          </Button>
          <Button
            variant="default"
            size="sm"
            onClick={handleApply}
            disabled={!currentProfile}
          >
            <Play className="h-4 w-4 mr-2" />
            {t('omp.actions.apply')}
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Config Status */}
        <ConfigStatus status={configStatus} />

        {/* Profile Selector */}
        <div className="space-y-3">
          <div className="flex items-center gap-2">
            <Label className="w-24 shrink-0">{t('omp.profile.select')}</Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={handleProfileChange}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('omp.profile.select')} />
              </SelectTrigger>
              <SelectContent>
                {profiles.map(profile => (
                  <SelectItem key={profile.id} value={profile.id}>
                    {profile.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowCreateDialog(true)}
              title={t('omp.profile.create')}
            >
              <Plus className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => {
                if (currentProfile) {
                  duplicateProfile(
                    currentProfile.id,
                    `${currentProfile.name} (copy)`
                  )
                }
              }}
              disabled={!currentProfile}
              title={t('omp.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => {
                if (currentProfile) {
                  deleteProfile(currentProfile.id)
                }
              }}
              disabled={!currentProfile || profiles.length <= 1}
              title={t('omp.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {/* Profile Name & Description */}
          {currentProfile && (
            <ProfileFields
              key={currentProfile.id}
              profile={currentProfile}
              onSave={updateCurrentProfile}
              t={t}
            />
          )}
        </div>

        {/* Model Roles */}
        {currentProfile && (
          <Card>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <CardTitle className="text-base">
                  {t('omp.modelRoles.title')}
                </CardTitle>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleLoadFromLive}
                  disabled={!configStatus?.configExists}
                >
                  <Download className="h-4 w-4 mr-1" />
                  {t('omp.actions.loadFromLive')}
                </Button>
              </div>
            </CardHeader>
            <CardContent className="space-y-3">
              {ROLE_KEYS.map(role => (
                <div key={role} className="flex items-center gap-2">
                  <Label className="w-24 shrink-0 text-sm capitalize">
                    {role}
                  </Label>
                  <Input
                    value={
                      (currentProfile.modelRoles as Record<string, string>)[
                        role
                      ] ?? ''
                    }
                    onChange={e => handleModelRoleChange(role, e.target.value)}
                    placeholder="provider/model-id"
                    className="flex-1 font-mono text-sm"
                  />
                </div>
              ))}
            </CardContent>
          </Card>
        )}

        {/* Providers */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base">
                {t('omp.providers.title')}
              </CardTitle>
              <Button variant="outline" size="sm" onClick={openNewProvider}>
                <Plus className="h-4 w-4 mr-1" />
                {t('omp.provider.add')}
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-2">
            <TooltipProvider>
              {customProviders.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  {t('omp.provider.noProviders')}
                </p>
              ) : (
                customProviders.map(entry => (
                  <ProviderCard
                    key={entry.providerId}
                    providerId={entry.providerId}
                    modelCount={entry.config.models?.length ?? 0}
                    hasApiKey={Boolean(entry.config.apiKey)}
                    onEdit={() => openEditProvider(entry)}
                    onDelete={() => setProviderToDelete(entry.providerId)}
                    onImport={() => openEditProvider(entry)}
                  />
                ))
              )}
            </TooltipProvider>
          </CardContent>
        </Card>

        {/* Live Config Preview */}
        {liveConfig && (
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">
                {t('omp.liveConfig.title')}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              {liveConfig.agentConfig?.modelRoles && (
                <div>
                  <span className="font-medium">
                    {t('omp.liveConfig.currentRoles')}:
                  </span>
                  <pre className="mt-1 p-2 bg-muted rounded text-xs overflow-auto">
                    {JSON.stringify(liveConfig.agentConfig.modelRoles, null, 2)}
                  </pre>
                </div>
              )}
            </CardContent>
          </Card>
        )}
      </div>

      {/* Create Profile Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('omp.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('omp.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <Input
            value={newProfileName}
            onChange={e => setNewProfileName(e.target.value)}
            placeholder={t('omp.profile.namePlaceholder')}
            onKeyDown={e => {
              if (e.key === 'Enter') handleCreateProfile()
            }}
          />
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowCreateDialog(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleCreateProfile}
              disabled={!newProfileName.trim()}
            >
              {t('common.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <OmpProviderEditor
        key={`${providerEditorOpen}-${editingProvider?.providerId ?? 'new'}`}
        open={providerEditorOpen}
        onOpenChange={setProviderEditorOpen}
        entry={editingProvider}
        onSave={handleSaveProvider}
      />
      <Dialog
        open={providerToDelete !== null}
        onOpenChange={open => {
          if (!open) setProviderToDelete(null)
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('omp.provider.delete')}</DialogTitle>
            <DialogDescription>
              {t('omp.provider.deleteDescription')}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setProviderToDelete(null)}>
              {t('common.cancel')}
            </Button>
            <Button variant="destructive" onClick={handleDeleteProvider}>
              {t('common.delete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
