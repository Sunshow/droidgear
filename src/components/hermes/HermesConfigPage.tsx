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
import { ConfigStatus } from './ConfigStatus'
import { ImportFromChannelDialog } from './ImportFromChannelDialog'

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

  // Local editing state for profile fields and model fields
  const profileKey = currentProfile?.id ?? ''
  const [editingName, setEditingName] = useState(currentProfile?.name ?? '')
  const [editingDescription, setEditingDescription] = useState(
    currentProfile?.description ?? ''
  )
  const [editingDefaultModel, setEditingDefaultModel] = useState(
    currentProfile?.model.default ?? ''
  )
  const [editingProvider, setEditingProvider] = useState(
    currentProfile?.model.provider ?? ''
  )
  const [editingBaseUrl, setEditingBaseUrl] = useState(
    currentProfile?.model.baseUrl ?? ''
  )
  const [editingApiKey, setEditingApiKey] = useState(
    currentProfile?.model.apiKey ?? ''
  )

  // Reset local state when profile changes
  const [lastProfileKey, setLastProfileKey] = useState(profileKey)
  if (profileKey !== lastProfileKey) {
    setLastProfileKey(profileKey)
    setEditingName(currentProfile?.name ?? '')
    setEditingDescription(currentProfile?.description ?? '')
    setEditingDefaultModel(currentProfile?.model.default ?? '')
    setEditingProvider(currentProfile?.model.provider ?? '')
    setEditingBaseUrl(currentProfile?.model.baseUrl ?? '')
    setEditingApiKey(currentProfile?.model.apiKey ?? '')
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
      setEditingDefaultModel(updated.model.default ?? '')
      setEditingProvider(updated.model.provider ?? '')
      setEditingBaseUrl(updated.model.baseUrl ?? '')
      setEditingApiKey(updated.model.apiKey ?? '')
    }
    toast.success(t('hermes.actions.loadedFromLive'))
  }

  const handleImportFromChannel = async (result: {
    baseUrl: string
    apiKey: string
    provider: string
    defaultModel?: string
  }) => {
    await importFromChannel(result)
    setEditingBaseUrl(result.baseUrl)
    setEditingApiKey(result.apiKey)
    setEditingProvider(result.provider)
    setEditingDefaultModel(result.defaultModel ?? '')
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

  const handleModelBlur = async () => {
    if (!currentProfile) return
    const updated = {
      ...currentProfile,
      model: {
        default: editingDefaultModel || null,
        provider: editingProvider || null,
        baseUrl: editingBaseUrl || null,
        apiKey: editingApiKey || null,
      },
      updatedAt: new Date().toISOString(),
    }
    useHermesStore.setState(
      { currentProfile: updated },
      undefined,
      'hermes/updateModelFields'
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
            <Label className="w-24">{t('hermes.profile.select')}</Label>
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
                <Label className="w-24">{t('hermes.profile.name')}</Label>
                <Input
                  value={editingName}
                  onChange={e => setEditingName(e.target.value)}
                  onBlur={handleProfileFieldBlur}
                  placeholder={t('hermes.profile.namePlaceholder')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-24">
                  {t('hermes.profile.description')}
                </Label>
                <Input
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

            <div className="space-y-3">
              <div className="flex items-center gap-2">
                <Label className="w-24">{t('hermes.model.default')}</Label>
                <Input
                  value={editingDefaultModel}
                  onChange={e => setEditingDefaultModel(e.target.value)}
                  onBlur={handleModelBlur}
                  placeholder={t('hermes.model.defaultPlaceholder')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-24">{t('hermes.model.provider')}</Label>
                <Input
                  value={editingProvider}
                  onChange={e => setEditingProvider(e.target.value)}
                  onBlur={handleModelBlur}
                  placeholder={t('hermes.model.providerPlaceholder')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-24">{t('hermes.model.baseUrl')}</Label>
                <Input
                  value={editingBaseUrl}
                  onChange={e => setEditingBaseUrl(e.target.value)}
                  onBlur={handleModelBlur}
                  placeholder={t('hermes.model.baseUrlPlaceholder')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-24">{t('hermes.model.apiKey')}</Label>
                <Input
                  type="password"
                  value={editingApiKey}
                  onChange={e => setEditingApiKey(e.target.value)}
                  onBlur={handleModelBlur}
                  placeholder={t('hermes.model.apiKeyPlaceholder')}
                />
              </div>
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
              {t('hermes.actions.applyConfirm')}
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
