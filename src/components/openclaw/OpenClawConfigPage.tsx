import { useState, useEffect, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Plus,
  AlertCircle,
  RefreshCw,
  Play,
  Copy,
  Trash2,
  Download,
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
import { useOpenClawStore } from '@/store/openclaw-store'
import { ProviderCard } from './ProviderCard'
import { ProviderDialog } from './ProviderDialog'

export function OpenClawConfigPage() {
  const { t } = useTranslation()
  const profiles = useOpenClawStore(state => state.profiles)
  const activeProfileId = useOpenClawStore(state => state.activeProfileId)
  const currentProfile = useOpenClawStore(state => state.currentProfile)
  const isLoading = useOpenClawStore(state => state.isLoading)
  const error = useOpenClawStore(state => state.error)
  const configStatus = useOpenClawStore(state => state.configStatus)

  const loadProfiles = useOpenClawStore(state => state.loadProfiles)
  const loadActiveProfileId = useOpenClawStore(
    state => state.loadActiveProfileId
  )
  const loadConfigStatus = useOpenClawStore(state => state.loadConfigStatus)
  const selectProfile = useOpenClawStore(state => state.selectProfile)
  const createProfile = useOpenClawStore(state => state.createProfile)
  const deleteProfile = useOpenClawStore(state => state.deleteProfile)
  const duplicateProfile = useOpenClawStore(state => state.duplicateProfile)
  const applyProfile = useOpenClawStore(state => state.applyProfile)
  const loadFromLiveConfig = useOpenClawStore(state => state.loadFromLiveConfig)
  const updateProfileName = useOpenClawStore(state => state.updateProfileName)
  const updateProfileDescription = useOpenClawStore(
    state => state.updateProfileDescription
  )
  const updateDefaultModel = useOpenClawStore(state => state.updateDefaultModel)
  const deleteProvider = useOpenClawStore(state => state.deleteProvider)
  const setError = useOpenClawStore(state => state.setError)

  const [providerDialogOpen, setProviderDialogOpen] = useState(false)
  const [editingProviderId, setEditingProviderId] = useState<string | null>(
    null
  )
  const [deleteProviderId, setDeleteProviderId] = useState<string | null>(null)
  const [showApplyConfirm, setShowApplyConfirm] = useState(false)
  const [showDeleteProfileConfirm, setShowDeleteProfileConfirm] =
    useState(false)
  const [showCreateProfileDialog, setShowCreateProfileDialog] = useState(false)
  const [showDuplicateProfileDialog, setShowDuplicateProfileDialog] =
    useState(false)
  const [newProfileName, setNewProfileName] = useState('')

  // Use profile id as key to reset local editing state
  const profileKey = currentProfile?.id ?? ''
  const [editingName, setEditingName] = useState(currentProfile?.name ?? '')
  const [editingDescription, setEditingDescription] = useState(
    currentProfile?.description ?? ''
  )

  // Reset local state when profile changes
  const [lastProfileKey, setLastProfileKey] = useState(profileKey)
  if (profileKey !== lastProfileKey) {
    setLastProfileKey(profileKey)
    setEditingName(currentProfile?.name ?? '')
    setEditingDescription(currentProfile?.description ?? '')
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
    toast.success(t('openclaw.actions.applySuccess'))
  }

  const handleAddProvider = () => {
    setEditingProviderId(null)
    setProviderDialogOpen(true)
  }

  const handleEditProvider = (providerId: string) => {
    setEditingProviderId(providerId)
    setProviderDialogOpen(true)
  }

  const handleConfirmDeleteProvider = () => {
    if (deleteProviderId) {
      deleteProvider(deleteProviderId)
      setDeleteProviderId(null)
    }
  }

  const handleLoadFromConfig = async () => {
    await loadFromLiveConfig()
    toast.success(t('openclaw.actions.loadedFromLive'))
  }

  const providerEntries = currentProfile
    ? Object.entries(currentProfile.providers ?? {})
    : []

  // Build available model refs from all configured providers/models
  const availableModelRefs = useMemo(() => {
    if (!currentProfile) return []
    return Object.entries(currentProfile.providers ?? {}).flatMap(
      ([providerId, config]) =>
        (config?.models ?? []).map(m => `${providerId}/${m.id}`)
    )
  }, [currentProfile])

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('openclaw.title')}</h1>
          <div className="flex items-center gap-2 mt-1">
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('openclaw.profile.active')}</Badge>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-shrink-0">
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
            {t('openclaw.actions.apply')}
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
            <Label className="w-20">{t('openclaw.profile.select')}</Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={handleProfileChange}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('openclaw.profile.select')} />
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
              title={t('openclaw.profile.create')}
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
              title={t('openclaw.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowDeleteProfileConfirm(true)}
              disabled={!currentProfile || profiles.length <= 1}
              title={t('openclaw.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {currentProfile && (
            <>
              <div className="flex items-center gap-2">
                <Label className="w-20">{t('openclaw.profile.name')}</Label>
                <Input
                  value={editingName}
                  onChange={e => setEditingName(e.target.value)}
                  onBlur={() => {
                    if (editingName !== currentProfile.name) {
                      updateProfileName(editingName)
                    }
                  }}
                  placeholder={t('openclaw.profile.name')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-20">
                  {t('openclaw.profile.description')}
                </Label>
                <Input
                  value={editingDescription}
                  onChange={e => setEditingDescription(e.target.value)}
                  onBlur={() => {
                    if (
                      editingDescription !== (currentProfile.description ?? '')
                    ) {
                      updateProfileDescription(editingDescription)
                    }
                  }}
                  placeholder={t('openclaw.profile.descriptionPlaceholder')}
                />
              </div>
            </>
          )}
        </div>

        {/* Default Model Section */}
        <div className="space-y-3 p-4 border rounded-lg">
          <h2 className="text-lg font-medium">
            {t('openclaw.defaultModel.title')}
          </h2>
          <div className="flex items-center gap-2">
            <Label className="w-32">{t('openclaw.defaultModel.label')}</Label>
            <Select
              value={currentProfile?.defaultModel ?? ''}
              onValueChange={updateDefaultModel}
              disabled={!currentProfile || availableModelRefs.length === 0}
            >
              <SelectTrigger className="flex-1">
                <SelectValue
                  placeholder={
                    availableModelRefs.length === 0
                      ? t('openclaw.defaultModel.noModelsConfigured')
                      : t('openclaw.defaultModel.selectPlaceholder')
                  }
                />
              </SelectTrigger>
              <SelectContent>
                {availableModelRefs.map(ref => (
                  <SelectItem key={ref} value={ref}>
                    {ref}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <p className="text-xs text-muted-foreground">
            {t('openclaw.defaultModel.hint')}
          </p>
        </div>

        {/* Providers Section */}
        <div className="space-y-3 p-4 border rounded-lg">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-medium">
              {t('openclaw.features.providers')}
            </h2>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleLoadFromConfig}
                disabled={!currentProfile || !configStatus?.configExists}
                title={t('openclaw.provider.loadFromConfig')}
              >
                <Download className="h-4 w-4 mr-2" />
                {t('openclaw.provider.load')}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleAddProvider}
                disabled={!currentProfile}
              >
                <Plus className="h-4 w-4 mr-2" />
                {t('openclaw.provider.add')}
              </Button>
            </div>
          </div>

          {providerEntries.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              {t('openclaw.provider.noProviders')}
            </div>
          ) : (
            <div className="space-y-2">
              {providerEntries.map(([providerId, config]) => (
                <ProviderCard
                  key={providerId}
                  providerId={providerId}
                  config={config}
                  onEdit={() => handleEditProvider(providerId)}
                  onDelete={() => setDeleteProviderId(providerId)}
                />
              ))}
            </div>
          )}
        </div>

        {/* Config Status */}
        {configStatus && (
          <div className="p-4 border rounded-lg">
            <h2 className="text-lg font-medium mb-2">
              {t('openclaw.configStatus.title')}
            </h2>
            <div className="text-sm text-muted-foreground space-y-1">
              <div className="flex items-center gap-2">
                <span>{t('openclaw.configStatus.configPath')}:</span>
                <code className="text-xs bg-muted px-1 py-0.5 rounded">
                  {configStatus.configPath}
                </code>
                <Badge
                  variant={configStatus.configExists ? 'default' : 'outline'}
                >
                  {configStatus.configExists
                    ? t('common.exists')
                    : t('common.missing')}
                </Badge>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Provider Dialog */}
      <ProviderDialog
        open={providerDialogOpen}
        onOpenChange={setProviderDialogOpen}
        editingProviderId={editingProviderId}
        currentProfile={currentProfile}
      />

      {/* Apply Confirmation */}
      <AlertDialog open={showApplyConfirm} onOpenChange={setShowApplyConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('openclaw.actions.apply')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('openclaw.actions.applyConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleApply}>
              {t('openclaw.actions.apply')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Delete Provider Confirmation */}
      <AlertDialog
        open={deleteProviderId !== null}
        onOpenChange={() => setDeleteProviderId(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('openclaw.provider.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('openclaw.provider.deleteConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmDeleteProvider}>
              {t('common.delete')}
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
            <AlertDialogTitle>{t('openclaw.profile.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('openclaw.profile.deleteConfirm')}
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
            <DialogTitle>{t('openclaw.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('openclaw.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('openclaw.profile.namePlaceholder')}
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
            <DialogTitle>{t('openclaw.profile.duplicate')}</DialogTitle>
            <DialogDescription>
              {t('openclaw.profile.duplicateDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('openclaw.profile.namePlaceholder')}
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
              {t('openclaw.profile.duplicate')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
