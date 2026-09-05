import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Plus,
  AlertCircle,
  RefreshCw,
  Play,
  Copy,
  Trash2,
  Download,
  CloudDownload,
  Circle,
  CheckCircle2,
} from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useHermesStore } from '@/store/hermes-store'
import { trimToNull } from '@/lib/utils'
import type { HermesModelConfig } from '@/lib/bindings'
import { ConfigStatus } from './ConfigStatus'
import { ImportFromChannelDialog } from './ImportFromChannelDialog'

function cleanEntry(entry: HermesModelConfig): HermesModelConfig {
  return {
    name: trimToNull(entry.name ?? ''),
    default: trimToNull(entry.default ?? ''),
    provider: trimToNull(entry.provider ?? ''),
    baseUrl: trimToNull(entry.baseUrl ?? ''),
    apiKey: trimToNull(entry.apiKey ?? ''),
    isDefault: entry.isDefault === true,
  }
}

function emptyEntry(isDefault: boolean): HermesModelConfig {
  return {
    name: null,
    default: null,
    provider: null,
    baseUrl: null,
    apiKey: null,
    isDefault,
  }
}

export function HermesConfigPage() {
  const { t } = useTranslation()
  const profiles = useHermesStore(state => state.profiles)
  const activeProfileId = useHermesStore(state => state.activeProfileId)
  const currentProfile = useHermesStore(state => state.currentProfile)
  const isLoading = useHermesStore(state => state.isLoading)
  const error = useHermesStore(state => state.error)
  const configStatus = useHermesStore(state => state.configStatus)

  const loadProfiles = useHermesStore(state => state.loadProfiles)
  const loadActiveProfileId = useHermesStore(state => state.loadActiveProfileId)
  const loadConfigStatus = useHermesStore(state => state.loadConfigStatus)
  const selectProfile = useHermesStore(state => state.selectProfile)
  const createProfile = useHermesStore(state => state.createProfile)
  const deleteProfile = useHermesStore(state => state.deleteProfile)
  const duplicateProfile = useHermesStore(state => state.duplicateProfile)
  const applyProfile = useHermesStore(state => state.applyProfile)
  const loadFromLiveConfig = useHermesStore(state => state.loadFromLiveConfig)
  const importFromChannel = useHermesStore(state => state.importFromChannel)
  const saveProfile = useHermesStore(state => state.saveProfile)
  const setError = useHermesStore(state => state.setError)

  const [showApplyConfirm, setShowApplyConfirm] = useState(false)
  const [showDeleteProfileConfirm, setShowDeleteProfileConfirm] =
    useState(false)
  const [showCreateProfileDialog, setShowCreateProfileDialog] = useState(false)
  const [showDuplicateProfileDialog, setShowDuplicateProfileDialog] =
    useState(false)
  const [showImportFromChannelDialog, setShowImportFromChannelDialog] =
    useState(false)
  const [newProfileName, setNewProfileName] = useState('')

  // Local editing state for profile fields and model list
  const profileKey = currentProfile?.id ?? ''
  const [editingName, setEditingName] = useState(currentProfile?.name ?? '')
  const [editingDescription, setEditingDescription] = useState(
    currentProfile?.description ?? ''
  )
  const [editingModels, setEditingModels] = useState<HermesModelConfig[]>(() =>
    (currentProfile?.models ?? []).map(m => ({ ...m }))
  )
  const [editingReasoningEffort, setEditingReasoningEffort] = useState(
    currentProfile?.reasoningEffort ?? ''
  )

  // Reset local state when profile changes
  const [lastProfileKey, setLastProfileKey] = useState(profileKey)
  if (profileKey !== lastProfileKey) {
    setLastProfileKey(profileKey)
    setEditingName(currentProfile?.name ?? '')
    setEditingDescription(currentProfile?.description ?? '')
    setEditingModels((currentProfile?.models ?? []).map(m => ({ ...m })))
    setEditingReasoningEffort(currentProfile?.reasoningEffort ?? '')
  }

  useEffect(() => {
    const init = async () => {
      await loadProfiles()
      await loadActiveProfileId()
    }
    init()
    loadConfigStatus()
  }, [loadProfiles, loadActiveProfileId, loadConfigStatus])

  const handleProfileChange = (profileId: string) => {
    selectProfile(profileId)
  }

  const handleCreateProfile = async () => {
    if (!newProfileName.trim()) return
    await createProfile(newProfileName.trim())
    setNewProfileName('')
    setShowCreateProfileDialog(false)
  }

  const handleDuplicateProfile = async () => {
    if (!currentProfile || !newProfileName.trim()) return
    await duplicateProfile(currentProfile.id, newProfileName.trim())
    setNewProfileName('')
    setShowDuplicateProfileDialog(false)
  }

  const handleDeleteProfile = async () => {
    if (!currentProfile) return
    await deleteProfile(currentProfile.id)
    setShowDeleteProfileConfirm(false)
  }

  const handleApply = async () => {
    if (!currentProfile) return
    await applyProfile(currentProfile.id)
    setShowApplyConfirm(false)
    toast.success(t('hermes.actions.applySuccess'))
  }

  const handleLoadFromConfig = async () => {
    await loadFromLiveConfig()
    // Sync local editing state from the updated currentProfile in store,
    // because the profile id doesn't change so the profileKey diff won't trigger.
    const updated = useHermesStore.getState().currentProfile
    if (updated) {
      setEditingModels((updated.models ?? []).map(m => ({ ...m })))
      setEditingReasoningEffort(updated.reasoningEffort ?? '')
    }
    toast.success(t('hermes.actions.loadedFromLive'))
  }

  const persistModelChanges = async (
    models: HermesModelConfig[],
    effort: string
  ) => {
    if (!currentProfile) return
    const updated = {
      ...currentProfile,
      models: models.map(cleanEntry),
      reasoningEffort: trimToNull(effort),
      updatedAt: new Date().toISOString(),
    }
    useHermesStore.setState(
      { currentProfile: updated },
      undefined,
      'hermes/updateModelFields'
    )
    await saveProfile()
  }

  const handleModelBlur = () => {
    persistModelChanges(editingModels, editingReasoningEffort)
  }

  const handleAddModel = () => {
    const next = [...editingModels, emptyEntry(editingModels.length === 0)]
    setEditingModels(next)
    persistModelChanges(next, editingReasoningEffort)
  }

  const handleRemoveModel = (index: number) => {
    const removed = editingModels[index]
    const next = editingModels.filter((_, i) => i !== index)
    if (removed?.isDefault && next.length > 0) {
      next[0] = { ...next[0], isDefault: true }
    }
    setEditingModels(next)
    persistModelChanges(next, editingReasoningEffort)
  }

  const handleSetDefault = (index: number) => {
    const next = editingModels.map((m, i) => ({
      ...m,
      isDefault: i === index,
    }))
    setEditingModels(next)
    persistModelChanges(next, editingReasoningEffort)
  }

  const handleImportFromChannel = async (result: {
    baseUrl: string
    apiKey: string
    provider: string
    name?: string
    defaultModel?: string
  }) => {
    await importFromChannel(result)
    // Sync editing state from the store (import appends a new default entry)
    const updated = useHermesStore.getState().currentProfile
    if (updated) {
      setEditingModels((updated.models ?? []).map(m => ({ ...m })))
    }
    toast.success(t('hermes.model.importDialog.imported'))
  }

  const handleProfileFieldBlur = async () => {
    if (!currentProfile) return
    const nameChanged = editingName !== currentProfile.name
    const descChanged =
      editingDescription !== (currentProfile.description ?? '')
    if (!nameChanged && !descChanged) return
    const updated = {
      ...currentProfile,
      name: editingName || currentProfile.name,
      description: editingDescription || null,
      updatedAt: new Date().toISOString(),
    }
    useHermesStore.setState(
      { currentProfile: updated },
      undefined,
      'hermes/updateProfileFields'
    )
    await saveProfile()
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('hermes.title')}</h1>
          <div className="flex items-center gap-2 mt-1">
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('hermes.profile.active')}</Badge>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Button
            variant="outline"
            size="icon"
            onClick={() => {
              loadProfiles()
              loadConfigStatus()
            }}
            disabled={isLoading}
            title={t('common.refresh')}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button
            onClick={() => setShowApplyConfirm(true)}
            disabled={!currentProfile || isLoading}
          >
            <Play className="h-4 w-4 mr-2" />
            {t('hermes.actions.apply')}
          </Button>
        </div>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md flex items-center gap-2">
          <AlertCircle className="h-4 w-4 text-destructive" />
          <span className="text-sm text-destructive">{error}</span>
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto"
            onClick={() => setError(null)}
          >
            {t('common.dismiss')}
          </Button>
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Profile Section */}
        <div className="space-y-3 p-4 border rounded-lg">
          <div className="flex items-center gap-2">
            <Label className="w-24 shrink-0">
              {t('hermes.profile.select')}
            </Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={handleProfileChange}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('hermes.profile.select')} />
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
              onClick={() => setShowCreateProfileDialog(true)}
              title={t('hermes.profile.create')}
            >
              <Plus className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => {
                setNewProfileName(
                  currentProfile?.name ? `${currentProfile.name} (Copy)` : ''
                )
                setShowDuplicateProfileDialog(true)
              }}
              disabled={!currentProfile}
              title={t('hermes.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowDeleteProfileConfirm(true)}
              disabled={!currentProfile || profiles.length <= 1}
              title={t('hermes.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {currentProfile && (
            <>
              <div className="flex items-center gap-2">
                <Label className="w-24 shrink-0">
                  {t('hermes.profile.name')}
                </Label>
                <Input
                  className="flex-1"
                  value={editingName}
                  onChange={e => setEditingName(e.target.value)}
                  onBlur={handleProfileFieldBlur}
                  placeholder={t('hermes.profile.namePlaceholder')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-24 shrink-0">
                  {t('hermes.profile.description')}
                </Label>
                <Input
                  className="flex-1"
                  value={editingDescription}
                  onChange={e => setEditingDescription(e.target.value)}
                  onBlur={handleProfileFieldBlur}
                  placeholder={t('hermes.profile.descriptionPlaceholder')}
                />
              </div>
            </>
          )}
        </div>

        {/* Model Config Section */}
        {currentProfile && (
          <div className="space-y-3 p-4 border rounded-lg">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-medium">{t('hermes.model.title')}</h2>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setShowImportFromChannelDialog(true)}
                  title={t('hermes.model.importFromChannel')}
                >
                  <CloudDownload className="h-4 w-4 mr-2" />
                  {t('hermes.model.importFromChannel')}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleLoadFromConfig}
                  disabled={!configStatus?.configExists}
                  title={t('hermes.model.loadFromConfig')}
                >
                  <Download className="h-4 w-4 mr-2" />
                  {t('hermes.model.loadFromConfig')}
                </Button>
              </div>
            </div>

            <p className="text-xs text-muted-foreground">
              {t('hermes.model.defaultHint')}
            </p>

            {editingModels.length === 0 && (
              <p className="text-sm text-muted-foreground py-2">
                {t('hermes.model.empty')}
              </p>
            )}

            <div className="space-y-3">
              {editingModels.map((entry, index) => (
                <div key={index} className="space-y-2 border rounded-md p-3">
                  <div className="flex items-center gap-2">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="shrink-0"
                      onClick={() => handleSetDefault(index)}
                      title={
                        entry.isDefault
                          ? t('hermes.model.isDefault')
                          : t('hermes.model.setDefault')
                      }
                    >
                      {entry.isDefault ? (
                        <CheckCircle2 className="h-4 w-4 text-primary" />
                      ) : (
                        <Circle className="h-4 w-4 text-muted-foreground" />
                      )}
                    </Button>
                    <Input
                      className="w-40 shrink-0"
                      value={entry.name ?? ''}
                      onChange={e =>
                        setEditingModels(prev =>
                          prev.map((m, i) =>
                            i === index ? { ...m, name: e.target.value } : m
                          )
                        )
                      }
                      onBlur={handleModelBlur}
                      placeholder={t('hermes.model.namePlaceholder')}
                    />
                    <Input
                      className="flex-1"
                      value={entry.default ?? ''}
                      onChange={e =>
                        setEditingModels(prev =>
                          prev.map((m, i) =>
                            i === index ? { ...m, default: e.target.value } : m
                          )
                        )
                      }
                      onBlur={handleModelBlur}
                      placeholder={t('hermes.model.defaultPlaceholder')}
                    />
                    {entry.isDefault && (
                      <Badge
                        variant="outline"
                        className="shrink-0 text-primary"
                      >
                        {t('hermes.model.isDefault')}
                      </Badge>
                    )}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="shrink-0"
                      onClick={() => handleRemoveModel(index)}
                      title={t('hermes.model.removeEntry')}
                    >
                      <Trash2 className="h-4 w-4 text-muted-foreground" />
                    </Button>
                  </div>
                  <div className="flex items-center gap-2 pl-10">
                    <Input
                      className="flex-1"
                      value={entry.provider ?? ''}
                      onChange={e =>
                        setEditingModels(prev =>
                          prev.map((m, i) =>
                            i === index ? { ...m, provider: e.target.value } : m
                          )
                        )
                      }
                      onBlur={handleModelBlur}
                      placeholder={t('hermes.model.providerPlaceholder')}
                    />
                    <Input
                      className="flex-1"
                      value={entry.baseUrl ?? ''}
                      onChange={e =>
                        setEditingModels(prev =>
                          prev.map((m, i) =>
                            i === index ? { ...m, baseUrl: e.target.value } : m
                          )
                        )
                      }
                      onBlur={handleModelBlur}
                      placeholder={t('hermes.model.baseUrlPlaceholder')}
                    />
                    <Input
                      className="flex-1"
                      type="password"
                      value={entry.apiKey ?? ''}
                      onChange={e =>
                        setEditingModels(prev =>
                          prev.map((m, i) =>
                            i === index ? { ...m, apiKey: e.target.value } : m
                          )
                        )
                      }
                      onBlur={handleModelBlur}
                      placeholder={t('hermes.model.apiKeyPlaceholder')}
                    />
                  </div>
                </div>
              ))}
            </div>

            <Button variant="outline" size="sm" onClick={handleAddModel}>
              <Plus className="h-4 w-4 mr-2" />
              {t('hermes.model.addEntry')}
            </Button>

            <div className="flex items-center gap-2 pt-2">
              <Label className="w-24 shrink-0">
                {t('hermes.model.reasoningEffort')}
              </Label>
              <Select
                value={editingReasoningEffort || '(none)'}
                onValueChange={value => {
                  const newValue = value === '(none)' ? '' : value
                  setEditingReasoningEffort(newValue)
                  persistModelChanges(editingModels, newValue)
                }}
              >
                <SelectTrigger className="flex-1">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="(none)">(none)</SelectItem>
                  <SelectItem value="none">none</SelectItem>
                  <SelectItem value="minimal">minimal</SelectItem>
                  <SelectItem value="low">low</SelectItem>
                  <SelectItem value="medium">medium</SelectItem>
                  <SelectItem value="high">high</SelectItem>
                  <SelectItem value="xhigh">xhigh</SelectItem>
                  <SelectItem value="max">max</SelectItem>
                  <SelectItem value="ultra">ultra</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        )}

        {/* Config Status */}
        <ConfigStatus status={configStatus} />
      </div>

      {/* Apply Confirmation */}
      <AlertDialog open={showApplyConfirm} onOpenChange={setShowApplyConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('hermes.actions.apply')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('hermes.actions.applyConfirm')}{' '}
              {configStatus?.configPath && (
                <code className="text-xs break-all">
                  {configStatus.configPath}
                </code>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleApply}>
              {t('hermes.actions.apply')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Delete Profile Confirmation */}
      <AlertDialog
        open={showDeleteProfileConfirm}
        onOpenChange={setShowDeleteProfileConfirm}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('hermes.profile.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('hermes.profile.deleteConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteProfile}>
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Create Profile Dialog */}
      <Dialog
        open={showCreateProfileDialog}
        onOpenChange={setShowCreateProfileDialog}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('hermes.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('hermes.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('hermes.profile.namePlaceholder')}
              onKeyDown={e => e.key === 'Enter' && handleCreateProfile()}
            />
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowCreateProfileDialog(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleCreateProfile}
              disabled={!newProfileName.trim()}
            >
              {t('common.add')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Duplicate Profile Dialog */}
      <Dialog
        open={showDuplicateProfileDialog}
        onOpenChange={setShowDuplicateProfileDialog}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('hermes.profile.duplicate')}</DialogTitle>
            <DialogDescription>
              {t('hermes.profile.duplicateDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('hermes.profile.namePlaceholder')}
              onKeyDown={e => e.key === 'Enter' && handleDuplicateProfile()}
            />
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowDuplicateProfileDialog(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleDuplicateProfile}
              disabled={!newProfileName.trim()}
            >
              {t('hermes.profile.duplicate')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Import from Channel Dialog */}
      <ImportFromChannelDialog
        open={showImportFromChannelDialog}
        onOpenChange={setShowImportFromChannelDialog}
        onImported={handleImportFromChannel}
      />
    </div>
  )
}
