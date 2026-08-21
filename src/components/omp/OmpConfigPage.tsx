import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { Play, Plus, Copy, Trash2, Download, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
import type { OmpProfile } from '@/lib/bindings'

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
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <Label className="w-24">{t('omp.profile.name')}</Label>
        <Input
          value={name}
          onChange={e => setName(e.target.value)}
          onBlur={handleBlur}
          placeholder={t('omp.profile.namePlaceholder')}
        />
      </div>
      <div className="flex items-center gap-2">
        <Label className="w-24">{t('omp.profile.description')}</Label>
        <Input
          value={description}
          onChange={e => setDescription(e.target.value)}
          onBlur={handleBlur}
          placeholder={t('omp.profile.descriptionPlaceholder')}
        />
      </div>
    </div>
  )
}

export function OmpConfigPage() {
  const { t } = useTranslation()
  const profiles = useOmpStore(state => state.profiles)
  const activeProfileId = useOmpStore(state => state.activeProfileId)
  const currentProfile = useOmpStore(state => state.currentProfile)
  const configStatus = useOmpStore(state => state.configStatus)
  const liveConfig = useOmpStore(state => state.liveConfig)
  const loadProfiles = useOmpStore(state => state.loadProfiles)
  const loadActiveProfileId = useOmpStore(state => state.loadActiveProfileId)
  const loadConfigStatus = useOmpStore(state => state.loadConfigStatus)
  const loadLiveConfig = useOmpStore(state => state.loadLiveConfig)
  const selectProfile = useOmpStore(state => state.selectProfile)
  const createProfile = useOmpStore(state => state.createProfile)
  const saveProfile = useOmpStore(state => state.saveProfile)
  const deleteProfile = useOmpStore(state => state.deleteProfile)
  const duplicateProfile = useOmpStore(state => state.duplicateProfile)
  const applyProfile = useOmpStore(state => state.applyProfile)

  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [newProfileName, setNewProfileName] = useState('')

  useEffect(() => {
    void loadProfiles().then(() => {
      void loadActiveProfileId()
    })
    void loadConfigStatus()
    void loadLiveConfig()
  }, [loadProfiles, loadActiveProfileId, loadConfigStatus, loadLiveConfig])

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
        <div className="flex items-center gap-2">
          <Label className="w-24">{t('omp.profile.select')}</Label>
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
                  <Label className="w-24 text-sm capitalize">{role}</Label>
                  <Input
                    value={
                      (currentProfile.modelRoles as Record<string, string>)[
                        role
                      ] ?? ''
                    }
                    onChange={e => handleModelRoleChange(role, e.target.value)}
                    placeholder="provider/model-id"
                    className="font-mono text-sm"
                  />
                </div>
              ))}
            </CardContent>
          </Card>
        )}

        {/* Providers */}
        {liveConfig &&
          liveConfig.providerModels &&
          liveConfig.providerModels.length > 0 && (
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t('omp.providers.title')}
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <TooltipProvider>
                  {liveConfig.providerModels.map(pm => {
                    const cred = liveConfig.credentials?.find(
                      c => c.provider === pm.providerId
                    )
                    return (
                      <ProviderCard
                        key={pm.providerId}
                        providerId={pm.providerId}
                        modelCount={pm.models.length}
                        hasApiKey={cred?.hasKey ?? false}
                      />
                    )
                  })}
                </TooltipProvider>
              </CardContent>
            </Card>
          )}

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
    </div>
  )
}
