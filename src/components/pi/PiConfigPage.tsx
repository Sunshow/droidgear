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
import { usePiStore } from '@/store/pi-store'
import { ConfigStatus } from './ConfigStatus'
import { ProviderCard } from './ProviderCard'
import { ProviderDialog } from './ProviderDialog'
import {
  PiImportFromChannelDialog,
  type PiImportResult,
} from './PiImportFromChannelDialog'
import type { PiProviderConfig } from '@/lib/bindings'

export function PiConfigPage() {
  const { t } = useTranslation()
  const profiles = usePiStore(state => state.profiles)
  const activeProfileId = usePiStore(state => state.activeProfileId)
  const currentProfile = usePiStore(state => state.currentProfile)
  const isLoading = usePiStore(state => state.isLoading)
  const error = usePiStore(state => state.error)
  const configStatus = usePiStore(state => state.configStatus)

  const loadProfiles = usePiStore(state => state.loadProfiles)
  const loadActiveProfileId = usePiStore(state => state.loadActiveProfileId)
  const loadConfigStatus = usePiStore(state => state.loadConfigStatus)
  const selectProfile = usePiStore(state => state.selectProfile)
  const createProfile = usePiStore(state => state.createProfile)
  const deleteProfile = usePiStore(state => state.deleteProfile)
  const duplicateProfile = usePiStore(state => state.duplicateProfile)
  const applyProfile = usePiStore(state => state.applyProfile)
  const loadFromLiveConfig = usePiStore(state => state.loadFromLiveConfig)
  const deleteProvider = usePiStore(state => state.deleteProvider)
  const addProvider = usePiStore(state => state.addProvider)
  const saveProfile = usePiStore(state => state.saveProfile)
  const setError = usePiStore(state => state.setError)

  const [providerDialogOpen, setProviderDialogOpen] = useState(false)
  const [editingProviderId, setEditingProviderId] = useState<string | null>(
    null
  )
  const [deleteProviderId, setDeleteProviderId] = useState<string | null>(null)
  const [importFromChannelOpen, setImportFromChannelOpen] = useState(false)
  const [showApplyConfirm, setShowApplyConfirm] = useState(false)
  const [showDeleteProfileConfirm, setShowDeleteProfileConfirm] =
    useState(false)
  const [showCreateProfileDialog, setShowCreateProfileDialog] = useState(false)
  const [showDuplicateProfileDialog, setShowDuplicateProfileDialog] =
    useState(false)
  const [newProfileName, setNewProfileName] = useState('')

  // Local editing state for profile fields
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
    setError(null)
    await applyProfile(currentProfile.id)
    setShowApplyConfirm(false)
    // Check if an error was set during apply
    const currentError = usePiStore.getState().error
    if (!currentError) {
      toast.success(t('pi.actions.applySuccess'))
    }
  }

  const handleLoadFromConfig = async () => {
    setError(null)
    await loadFromLiveConfig()
    // Check if an error was set during load
    const currentError = usePiStore.getState().error
    if (!currentError) {
      toast.success(t('pi.actions.loadedFromLive'))
    }
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

  const handleImportFromChannel = async (result: PiImportResult) => {
    const config: PiProviderConfig = {
      baseUrl: result.baseUrl,
      api: result.api,
      apiKey: result.apiKey,
      headers: null,
      authHeader: null,
      models: result.models,
      modelOverrides: null,
      compat: null,
    }
    await addProvider(result.providerId, config)
    toast.success(t('pi.provider.importDialog.imported'))
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
    usePiStore.setState(
      { currentProfile: updated },
      undefined,
      'pi/updateProfileFields'
    )
    await saveProfile()
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('pi.title')}</h1>
          <div className="flex items-center gap-2 mt-1">
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('pi.profile.active')}</Badge>
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
            {t('pi.actions.apply')}
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
            <Label className="w-20">{t('pi.profile.select')}</Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={handleProfileChange}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('pi.profile.select')} />
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
              title={t('pi.profile.create')}
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
              title={t('pi.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowDeleteProfileConfirm(true)}
              disabled={!currentProfile || profiles.length <= 1}
              title={t('pi.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {currentProfile && (
            <>
              <div className="flex items-center gap-2">
                <Label className="w-24">{t('pi.profile.name')}</Label>
                <Input
                  value={editingName}
                  onChange={e => setEditingName(e.target.value)}
                  onBlur={handleProfileFieldBlur}
                  placeholder={t('pi.profile.namePlaceholder')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-24">{t('pi.profile.description')}</Label>
                <Input
                  value={editingDescription}
                  onChange={e => setEditingDescription(e.target.value)}
                  onBlur={handleProfileFieldBlur}
                  placeholder={t('pi.profile.descriptionPlaceholder')}
                />
              </div>
            </>
          )}
        </div>

        {/* Providers Section */}
        {currentProfile && (
          <div className="space-y-3 p-4 border rounded-lg">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-medium">{t('pi.providers.title')}</h2>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleLoadFromConfig}
                  disabled={!configStatus?.configExists}
                  title={t('pi.providers.loadFromConfig')}
                >
                  <Download className="h-4 w-4 mr-2" />
                  {t('pi.providers.loadFromConfig')}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setImportFromChannelOpen(true)}
                  disabled={!currentProfile}
                  title={t('pi.provider.importFromChannel')}
                >
                  <CloudDownload className="h-4 w-4 mr-2" />
                  {t('pi.provider.importFromChannel')}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleAddProvider}
                  disabled={!currentProfile}
                >
                  <Plus className="h-4 w-4 mr-2" />
                  {t('pi.provider.add')}
                </Button>
              </div>
            </div>

            {(() => {
              const providerEntries = currentProfile.providers
                ? Object.entries(currentProfile.providers)
                : []
              if (providerEntries.length === 0) {
                return (
                  <div className="text-center py-8 text-muted-foreground">
                    {t('pi.provider.noProviders')}
                  </div>
                )
              }
              return (
                <div className="space-y-2">
                  {providerEntries.map(([providerId, config]) => (
                    <ProviderCard
                      key={providerId}
                      providerId={providerId}
                      config={config ?? undefined}
                      onEdit={() => handleEditProvider(providerId)}
                      onDelete={() => setDeleteProviderId(providerId)}
                    />
                  ))}
                </div>
              )
            })()}
          </div>
        )}

        {/* Config Status */}
        <ConfigStatus status={configStatus} />
      </div>

      {/* Apply Confirmation */}
      <AlertDialog open={showApplyConfirm} onOpenChange={setShowApplyConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('pi.actions.apply')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('pi.actions.applyConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleApply}>
              {t('pi.actions.apply')}
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
            <AlertDialogTitle>{t('pi.profile.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('pi.profile.deleteConfirm')}
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
            <DialogTitle>{t('pi.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('pi.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('pi.profile.namePlaceholder')}
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
            <DialogTitle>{t('pi.profile.duplicate')}</DialogTitle>
            <DialogDescription>
              {t('pi.profile.duplicateDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('pi.profile.namePlaceholder')}
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
              {t('pi.profile.duplicate')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Provider Dialog */}
      <ProviderDialog
        open={providerDialogOpen}
        onOpenChange={setProviderDialogOpen}
        editingProviderId={editingProviderId}
        currentProfile={currentProfile}
      />

      {/* Import from Channel Dialog */}
      <PiImportFromChannelDialog
        open={importFromChannelOpen}
        onOpenChange={setImportFromChannelOpen}
        onImported={handleImportFromChannel}
        existingProviderIds={
          currentProfile?.providers ? Object.keys(currentProfile.providers) : []
        }
      />

      {/* Delete Provider Confirmation */}
      <AlertDialog
        open={deleteProviderId !== null}
        onOpenChange={() => setDeleteProviderId(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('pi.provider.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('pi.provider.deleteConfirm')}
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
    </div>
  )
}
