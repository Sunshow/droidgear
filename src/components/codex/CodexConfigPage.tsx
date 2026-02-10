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
import { useCodexStore } from '@/store/codex-store'
import type { CodexProviderConfig } from '@/lib/bindings'
import { ProviderCard } from './ProviderCard'
import { ProviderDialog } from './ProviderDialog'
import { ConfigStatus } from './ConfigStatus'

export function CodexConfigPage() {
  const { t } = useTranslation()
  const profiles = useCodexStore(state => state.profiles)
  const activeProfileId = useCodexStore(state => state.activeProfileId)
  const currentProfile = useCodexStore(state => state.currentProfile)
  const isLoading = useCodexStore(state => state.isLoading)
  const error = useCodexStore(state => state.error)
  const configStatus = useCodexStore(state => state.configStatus)

  const loadProfiles = useCodexStore(state => state.loadProfiles)
  const loadActiveProfileId = useCodexStore(state => state.loadActiveProfileId)
  const loadConfigStatus = useCodexStore(state => state.loadConfigStatus)
  const selectProfile = useCodexStore(state => state.selectProfile)
  const createProfile = useCodexStore(state => state.createProfile)
  const deleteProfile = useCodexStore(state => state.deleteProfile)
  const duplicateProfile = useCodexStore(state => state.duplicateProfile)
  const applyProfile = useCodexStore(state => state.applyProfile)
  const loadFromLiveConfig = useCodexStore(state => state.loadFromLiveConfig)
  const updateProfileName = useCodexStore(state => state.updateProfileName)
  const updateProfileDescription = useCodexStore(
    state => state.updateProfileDescription
  )
  const deleteProvider = useCodexStore(state => state.deleteProvider)
  const setActiveProvider = useCodexStore(state => state.setActiveProvider)
  const setError = useCodexStore(state => state.setError)

  const [showApplyConfirm, setShowApplyConfirm] = useState(false)
  const [showDeleteProfileConfirm, setShowDeleteProfileConfirm] =
    useState(false)
  const [showCreateProfileDialog, setShowCreateProfileDialog] = useState(false)
  const [showDuplicateProfileDialog, setShowDuplicateProfileDialog] =
    useState(false)
  const [newProfileName, setNewProfileName] = useState('')
  const [showProviderDialog, setShowProviderDialog] = useState(false)
  const [editingProviderId, setEditingProviderId] = useState<string | null>(
    null
  )
  const [showDeleteProviderConfirm, setShowDeleteProviderConfirm] =
    useState(false)
  const [deletingProviderId, setDeletingProviderId] = useState<string | null>(
    null
  )

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

  const providers = (currentProfile?.providers ?? {}) as Record<
    string,
    CodexProviderConfig
  >
  const providerIds = Object.keys(providers)

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
    toast.success(t('codex.actions.applySuccess'))
  }

  const handleLoadFromConfig = async () => {
    await loadFromLiveConfig()
    toast.success(t('codex.actions.loadedFromLive'))
  }

  const handleAddProvider = () => {
    setEditingProviderId(null)
    setShowProviderDialog(true)
  }

  const handleEditProvider = (providerId: string) => {
    setEditingProviderId(providerId)
    setShowProviderDialog(true)
  }

  const handleDeleteProviderClick = (providerId: string) => {
    setDeletingProviderId(providerId)
    setShowDeleteProviderConfirm(true)
  }

  const handleDeleteProviderConfirm = async () => {
    if (!deletingProviderId) return
    await deleteProvider(deletingProviderId)
    setShowDeleteProviderConfirm(false)
    setDeletingProviderId(null)
    toast.success(t('codex.provider.deleteSuccess'))
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('codex.title')}</h1>
          <div className="flex items-center gap-2 mt-1">
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('codex.profile.active')}</Badge>
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
            {t('codex.actions.apply')}
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
            <Label className="w-20">{t('codex.profile.select')}</Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={handleProfileChange}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('codex.profile.select')} />
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
              title={t('codex.profile.create')}
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
              title={t('codex.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowDeleteProfileConfirm(true)}
              disabled={!currentProfile || profiles.length <= 1}
              title={t('codex.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {currentProfile && (
            <>
              <div className="flex items-center gap-2">
                <Label className="w-20">{t('codex.profile.name')}</Label>
                <Input
                  value={editingName}
                  onChange={e => setEditingName(e.target.value)}
                  onBlur={() => {
                    if (editingName !== currentProfile.name) {
                      updateProfileName(editingName)
                    }
                  }}
                  placeholder={t('codex.profile.name')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-20">{t('codex.profile.description')}</Label>
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
                  placeholder={t('codex.profile.descriptionPlaceholder')}
                />
              </div>
            </>
          )}
        </div>

        {/* Providers Section */}
        {currentProfile && (
          <div className="space-y-3 p-4 border rounded-lg">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-medium">
                {t('codex.provider.title')}
              </h2>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleLoadFromConfig}
                  disabled={!configStatus?.configExists}
                  title={t('codex.provider.loadFromConfig')}
                >
                  <Download className="h-4 w-4 mr-2" />
                  {t('codex.provider.loadFromConfig')}
                </Button>
                <Button variant="outline" size="sm" onClick={handleAddProvider}>
                  <Plus className="h-4 w-4 mr-2" />
                  {t('codex.provider.add')}
                </Button>
              </div>
            </div>

            {providerIds.length === 0 ? (
              <div className="text-sm text-muted-foreground text-center py-6 border rounded-md">
                {t('codex.provider.noProviders')}
              </div>
            ) : (
              <div className="space-y-2">
                {providerIds.map(id => {
                  const config = providers[id]
                  if (!config) return null
                  return (
                    <ProviderCard
                      key={id}
                      providerId={id}
                      config={config}
                      isActive={currentProfile.modelProvider === id}
                      onEdit={() => handleEditProvider(id)}
                      onDelete={() => handleDeleteProviderClick(id)}
                      onSetActive={() => setActiveProvider(id)}
                    />
                  )
                })}
              </div>
            )}
          </div>
        )}

        {/* Config Status */}
        <ConfigStatus status={configStatus} />
      </div>

      {/* Apply Confirmation */}
      <AlertDialog open={showApplyConfirm} onOpenChange={setShowApplyConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('codex.actions.apply')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('codex.actions.applyConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleApply}>
              {t('codex.actions.apply')}
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
            <AlertDialogTitle>{t('codex.profile.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('codex.profile.deleteConfirm')}
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
            <DialogTitle>{t('codex.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('codex.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('codex.profile.namePlaceholder')}
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
            <DialogTitle>{t('codex.profile.duplicate')}</DialogTitle>
            <DialogDescription>
              {t('codex.profile.duplicateDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('codex.profile.namePlaceholder')}
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
              {t('codex.profile.duplicate')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Provider Dialog */}
      <ProviderDialog
        open={showProviderDialog}
        onOpenChange={setShowProviderDialog}
        editingProviderId={editingProviderId}
      />

      {/* Delete Provider Confirmation */}
      <AlertDialog
        open={showDeleteProviderConfirm}
        onOpenChange={setShowDeleteProviderConfirm}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('codex.provider.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('codex.provider.deleteConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteProviderConfirm}>
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
