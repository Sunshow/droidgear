import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Plus,
  Save,
  AlertCircle,
  RefreshCw,
  Play,
  Copy,
  Trash2,
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
import { useOpenCodeStore } from '@/store/opencode-store'
import { ProviderCard } from './ProviderCard'
import { ProviderDialog } from './ProviderDialog'
import { ConfigStatus } from './ConfigStatus'

export function OpenCodeConfigPage() {
  const { t } = useTranslation()
  const profiles = useOpenCodeStore(state => state.profiles)
  const activeProfileId = useOpenCodeStore(state => state.activeProfileId)
  const currentProfile = useOpenCodeStore(state => state.currentProfile)
  const hasChanges = useOpenCodeStore(state => state.hasChanges)
  const isLoading = useOpenCodeStore(state => state.isLoading)
  const error = useOpenCodeStore(state => state.error)
  const configStatus = useOpenCodeStore(state => state.configStatus)

  const loadProfiles = useOpenCodeStore(state => state.loadProfiles)
  const loadActiveProfileId = useOpenCodeStore(
    state => state.loadActiveProfileId
  )
  const loadConfigStatus = useOpenCodeStore(state => state.loadConfigStatus)
  const loadProviderTemplates = useOpenCodeStore(
    state => state.loadProviderTemplates
  )
  const selectProfile = useOpenCodeStore(state => state.selectProfile)
  const createProfile = useOpenCodeStore(state => state.createProfile)
  const saveProfile = useOpenCodeStore(state => state.saveProfile)
  const deleteProfile = useOpenCodeStore(state => state.deleteProfile)
  const duplicateProfile = useOpenCodeStore(state => state.duplicateProfile)
  const applyProfile = useOpenCodeStore(state => state.applyProfile)
  const updateProfileName = useOpenCodeStore(state => state.updateProfileName)
  const updateProfileDescription = useOpenCodeStore(
    state => state.updateProfileDescription
  )
  const deleteProvider = useOpenCodeStore(state => state.deleteProvider)
  const resetChanges = useOpenCodeStore(state => state.resetChanges)
  const setError = useOpenCodeStore(state => state.setError)

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

  useEffect(() => {
    loadProfiles()
    loadActiveProfileId()
    loadConfigStatus()
    loadProviderTemplates()
  }, [
    loadProfiles,
    loadActiveProfileId,
    loadConfigStatus,
    loadProviderTemplates,
  ])

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

  const handleSave = async () => {
    await saveProfile()
    toast.success(t('opencode.actions.saveSuccess'))
  }

  const handleApply = async () => {
    if (!currentProfile) return
    await applyProfile(currentProfile.id)
    setShowApplyConfirm(false)
    toast.success(t('opencode.actions.applySuccess'))
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

  const providerEntries = currentProfile
    ? Object.entries(currentProfile.providers)
    : []

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('opencode.title')}</h1>
          <div className="flex items-center gap-2 mt-1">
            {hasChanges && (
              <Badge
                variant="secondary"
                className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
              >
                {t('models.unsavedChanges')}
              </Badge>
            )}
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('opencode.profile.active')}</Badge>
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
            variant="outline"
            onClick={resetChanges}
            disabled={!hasChanges || isLoading}
          >
            {t('common.reset')}
          </Button>
          <Button
            variant="outline"
            onClick={handleSave}
            disabled={!hasChanges || isLoading}
          >
            <Save className="h-4 w-4 mr-2" />
            {t('common.save')}
          </Button>
          <Button
            onClick={() => setShowApplyConfirm(true)}
            disabled={!currentProfile || isLoading}
          >
            <Play className="h-4 w-4 mr-2" />
            {t('opencode.actions.apply')}
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
            <Label className="w-20">{t('opencode.profile.select')}</Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={handleProfileChange}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('opencode.profile.select')} />
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
              title={t('opencode.profile.create')}
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
              title={t('opencode.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowDeleteProfileConfirm(true)}
              disabled={!currentProfile || profiles.length <= 1}
              title={t('opencode.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {currentProfile && (
            <>
              <div className="flex items-center gap-2">
                <Label className="w-20">{t('opencode.profile.name')}</Label>
                <Input
                  value={currentProfile.name}
                  onChange={e => updateProfileName(e.target.value)}
                  placeholder={t('opencode.profile.name')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-20">
                  {t('opencode.profile.description')}
                </Label>
                <Input
                  value={currentProfile.description ?? ''}
                  onChange={e => updateProfileDescription(e.target.value)}
                  placeholder={t('opencode.profile.descriptionPlaceholder')}
                />
              </div>
            </>
          )}
        </div>

        {/* Providers Section */}
        <div className="space-y-3 p-4 border rounded-lg">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-medium">
              {t('opencode.features.providers')}
            </h2>
            <Button
              variant="outline"
              size="sm"
              onClick={handleAddProvider}
              disabled={!currentProfile}
            >
              <Plus className="h-4 w-4 mr-2" />
              {t('opencode.provider.add')}
            </Button>
          </div>

          {providerEntries.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              {t('opencode.provider.noProviders')}
            </div>
          ) : (
            <div className="space-y-2">
              {providerEntries.map(([providerId, config]) => (
                <ProviderCard
                  key={providerId}
                  providerId={providerId}
                  config={config}
                  auth={currentProfile?.auth[providerId]}
                  onEdit={() => handleEditProvider(providerId)}
                  onDelete={() => setDeleteProviderId(providerId)}
                />
              ))}
            </div>
          )}
        </div>

        {/* Config Status */}
        <ConfigStatus status={configStatus} />
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
            <AlertDialogTitle>{t('opencode.actions.apply')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('opencode.actions.applyConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleApply}>
              {t('opencode.actions.apply')}
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
            <AlertDialogTitle>{t('opencode.provider.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('opencode.provider.deleteConfirm')}
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
            <AlertDialogTitle>{t('opencode.profile.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('opencode.profile.deleteConfirm')}
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
            <DialogTitle>{t('opencode.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('opencode.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('opencode.profile.namePlaceholder')}
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
            <DialogTitle>{t('opencode.profile.duplicate')}</DialogTitle>
            <DialogDescription>
              {t('opencode.profile.duplicateDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('opencode.profile.namePlaceholder')}
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
              {t('opencode.profile.duplicate')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
