import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  AlertCircle,
  Copy,
  Download,
  Feather,
  Play,
  Plus,
  RefreshCw,
  Trash2,
} from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { SecretInput } from '@/components/ui/secret-input'
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
import {
  commands,
  type ClaudeCodeProfile,
  type ClaudeReasoningEffort,
  type ClaudeThinkingMode,
} from '@/lib/bindings'
import { hasOpaqueClaudeModelId } from '@/lib/utils'
import { useClaudeStore } from '@/store/claude-store'
import { ConfigStatus } from './ConfigStatus'

const REASONING_NONE = '__inherit__'

function patchCurrentProfile(patch: Partial<ClaudeCodeProfile>) {
  const currentProfile = useClaudeStore.getState().currentProfile
  if (!currentProfile) return
  useClaudeStore.setState(
    {
      currentProfile: {
        ...currentProfile,
        ...patch,
        updatedAt: new Date().toISOString(),
      },
    },
    undefined,
    'claude/patchCurrentProfile'
  )
}

export function ClaudeConfigPage() {
  const { t } = useTranslation()
  const profiles = useClaudeStore(state => state.profiles)
  const activeProfileId = useClaudeStore(state => state.activeProfileId)
  const currentProfile = useClaudeStore(state => state.currentProfile)
  const isLoading = useClaudeStore(state => state.isLoading)
  const error = useClaudeStore(state => state.error)
  const configStatus = useClaudeStore(state => state.configStatus)

  const loadProfiles = useClaudeStore(state => state.loadProfiles)
  const loadActiveProfileId = useClaudeStore(state => state.loadActiveProfileId)
  const loadConfigStatus = useClaudeStore(state => state.loadConfigStatus)
  const selectProfile = useClaudeStore(state => state.selectProfile)
  const createProfile = useClaudeStore(state => state.createProfile)
  const saveProfile = useClaudeStore(state => state.saveProfile)
  const deleteProfile = useClaudeStore(state => state.deleteProfile)
  const duplicateProfile = useClaudeStore(state => state.duplicateProfile)
  const applyProfile = useClaudeStore(state => state.applyProfile)
  const loadFromLiveConfig = useClaudeStore(state => state.loadFromLiveConfig)
  const setError = useClaudeStore(state => state.setError)

  const [showApplyConfirm, setShowApplyConfirm] = useState(false)
  const [showDeleteProfileConfirm, setShowDeleteProfileConfirm] =
    useState(false)
  const [showCreateProfileDialog, setShowCreateProfileDialog] = useState(false)
  const [showDuplicateProfileDialog, setShowDuplicateProfileDialog] =
    useState(false)
  const [newProfileName, setNewProfileName] = useState('')
  const [isLaunching, setIsLaunching] = useState(false)

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
    if (!useClaudeStore.getState().error) {
      toast.success(t('claude.actions.applySuccess'))
    }
  }

  const formatLaunchError = (message: string) => {
    if (message.startsWith('Failed to execute claude --version:')) {
      return t('claude.actions.launchMissingCli')
    }

    if (message === 'Failed to read Claude CLI version') {
      return t('claude.actions.launchInspectFailed')
    }

    return message
  }

  const handleLaunch = async () => {
    if (!currentProfile || isLaunching) return

    setIsLaunching(true)
    setError(null)

    try {
      await saveProfile()
      const saveState = useClaudeStore.getState()
      if (saveState.error) {
        toast.error(saveState.error)
        return
      }

      const preview = await commands.getClaudeTemporaryRunPlan(
        currentProfile.id
      )
      if (preview.status !== 'ok') {
        const message = formatLaunchError(preview.error)
        setError(message)
        toast.error(message)
        return
      }
      preview.data.warnings.forEach(warning => {
        toast.warning(warning)
      })

      const result = await commands.launchClaude(currentProfile.id)
      if (result.status === 'ok') {
        toast.success(t('claude.actions.launchSuccess'))
        return
      }

      const message = formatLaunchError(result.error)
      setError(message)
      toast.error(message)
    } finally {
      setIsLaunching(false)
    }
  }

  const handleLoadFromConfig = async () => {
    setError(null)
    await loadFromLiveConfig()
    if (!useClaudeStore.getState().error) {
      toast.success(t('claude.actions.loadedFromLive'))
    }
  }

  const saveCurrentProfile = async () => {
    const latest = useClaudeStore.getState().currentProfile
    if (!latest) return
    if (!latest.name.trim()) {
      patchCurrentProfile({ name: 'Default' })
    }
    await saveProfile()
  }

  const currentReasoning = currentProfile?.reasoningEffort ?? null
  const currentThinkingMode = currentProfile?.thinkingMode ?? 'inherit'
  const showOpaqueModelWarning = hasOpaqueClaudeModelId(currentProfile?.model)

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('claude.title')}</h1>
          <div className="flex items-center gap-2 mt-1">
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('claude.profile.active')}</Badge>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-shrink-0">
          <Button
            onClick={handleLaunch}
            disabled={!currentProfile || isLoading || isLaunching}
            title={t('claude.actions.launchTooltip')}
          >
            <Play className="h-4 w-4 mr-2" />
            {t('claude.actions.launch')}
          </Button>
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
            onClick={() => setShowApplyConfirm(true)}
            disabled={!currentProfile || isLoading || isLaunching}
          >
            {t('claude.actions.apply')}
          </Button>
        </div>
      </div>

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

      <div className="flex-1 overflow-auto p-4 space-y-4">
        <div className="space-y-3 p-4 border rounded-lg">
          <div className="flex items-center gap-2">
            <Label className="w-20">{t('claude.profile.select')}</Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={handleProfileChange}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('claude.profile.select')} />
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
              title={t('claude.profile.create')}
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
              title={t('claude.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowDeleteProfileConfirm(true)}
              disabled={!currentProfile}
              title={t('claude.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {currentProfile ? (
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="claude-name">{t('claude.profile.name')}</Label>
                <Input
                  id="claude-name"
                  value={currentProfile.name}
                  onChange={e => patchCurrentProfile({ name: e.target.value })}
                  onBlur={saveCurrentProfile}
                  placeholder={t('claude.profile.namePlaceholder')}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="claude-description">
                  {t('claude.profile.description')}
                </Label>
                <Input
                  id="claude-description"
                  value={currentProfile.description ?? ''}
                  onChange={e =>
                    patchCurrentProfile({ description: e.target.value || null })
                  }
                  onBlur={saveCurrentProfile}
                  placeholder={t('claude.profile.descriptionPlaceholder')}
                />
              </div>
            </div>
          ) : (
            <div className="text-sm text-muted-foreground">
              {t('claude.profile.noProfile')}
            </div>
          )}
        </div>

        {currentProfile && (
          <>
            <div className="space-y-4 p-4 border rounded-lg">
              <div className="flex items-center justify-between gap-2">
                <div>
                  <h2 className="text-sm font-medium">
                    {t('claude.provider.title')}
                  </h2>
                  <p className="text-xs text-muted-foreground mt-1">
                    {t('claude.provider.hint')}
                  </p>
                </div>
                <Button variant="outline" onClick={handleLoadFromConfig}>
                  <Download className="h-4 w-4 mr-2" />
                  {t('claude.actions.loadFromLive')}
                </Button>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="claude-base-url">
                    {t('claude.provider.baseUrl')}
                  </Label>
                  <Input
                    id="claude-base-url"
                    value={currentProfile.baseUrl ?? ''}
                    onChange={e =>
                      patchCurrentProfile({
                        baseUrl: e.target.value || null,
                      })
                    }
                    onBlur={saveCurrentProfile}
                    placeholder={t('claude.provider.baseUrlPlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="claude-bearer-token">
                    {t('claude.provider.bearerToken')}
                  </Label>
                  <SecretInput
                    id="claude-bearer-token"
                    value={currentProfile.bearerToken ?? ''}
                    onChange={e =>
                      patchCurrentProfile({
                        bearerToken: e.target.value || null,
                      })
                    }
                    onBlur={saveCurrentProfile}
                    placeholder={t('claude.provider.bearerTokenPlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="claude-model">{t('claude.model.name')}</Label>
                  <Input
                    id="claude-model"
                    value={currentProfile.model ?? ''}
                    onChange={e =>
                      patchCurrentProfile({ model: e.target.value || null })
                    }
                    onBlur={saveCurrentProfile}
                    placeholder={t('claude.model.namePlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="claude-small-model">
                    {t('claude.model.smallModel')}
                  </Label>
                  <Input
                    id="claude-small-model"
                    value={currentProfile.smallModel ?? ''}
                    onChange={e =>
                      patchCurrentProfile({
                        smallModel: e.target.value || null,
                      })
                    }
                    onBlur={saveCurrentProfile}
                    placeholder={t('claude.model.smallModelPlaceholder')}
                    disabled={currentProfile.smallModelUsesMainModel === true}
                  />
                </div>
              </div>

              {showOpaqueModelWarning && (
                <div className="flex items-start gap-3 rounded-md border border-amber-500/30 bg-amber-500/10 p-3">
                  <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-amber-600" />
                  <div className="space-y-1">
                    <p className="text-sm font-medium text-amber-700">
                      {t('claude.model.capabilityWarningTitle')}
                    </p>
                    <p className="text-xs text-amber-700/90">
                      {t('claude.model.capabilityWarningBody', {
                        model: currentProfile.model?.trim() ?? '',
                      })}
                    </p>
                  </div>
                </div>
              )}

              <div className="flex items-start gap-3 rounded-md border p-3">
                <Checkbox
                  id="claude-small-model-main"
                  checked={currentProfile.smallModelUsesMainModel === true}
                  onCheckedChange={async checked => {
                    patchCurrentProfile({
                      smallModelUsesMainModel: checked === true,
                    })
                    await saveCurrentProfile()
                  }}
                />
                <div className="space-y-1">
                  <Label
                    htmlFor="claude-small-model-main"
                    className="text-sm font-medium"
                  >
                    {t('claude.model.smallModelUsesMainModel')}
                  </Label>
                  <p className="text-xs text-muted-foreground">
                    {t('claude.model.smallModelUsesMainModelHint')}
                  </p>
                </div>
              </div>
            </div>

            <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_320px]">
              <div className="space-y-4 p-4 border rounded-lg">
                <div className="flex items-center gap-2">
                  <Feather className="h-4 w-4 text-muted-foreground" />
                  <h2 className="text-sm font-medium">
                    {t('claude.reasoning.title')}
                  </h2>
                </div>

                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <Label>{t('claude.reasoning.effort')}</Label>
                    <Select
                      value={currentReasoning ?? REASONING_NONE}
                      onValueChange={async value => {
                        patchCurrentProfile({
                          reasoningEffort:
                            value === REASONING_NONE
                              ? null
                              : (value as ClaudeReasoningEffort),
                        })
                        await saveCurrentProfile()
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue
                          placeholder={t('claude.reasoning.effortPlaceholder')}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value={REASONING_NONE}>
                          {t('claude.reasoning.effortInherit')}
                        </SelectItem>
                        <SelectItem value="low">
                          {t('claude.reasoning.low')}
                        </SelectItem>
                        <SelectItem value="medium">
                          {t('claude.reasoning.medium')}
                        </SelectItem>
                        <SelectItem value="high">
                          {t('claude.reasoning.high')}
                        </SelectItem>
                        <SelectItem value="max">
                          {t('claude.reasoning.max')}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <Label>{t('claude.reasoning.thinkingMode')}</Label>
                    <Select
                      value={currentThinkingMode}
                      onValueChange={async value => {
                        patchCurrentProfile({
                          thinkingMode: value as ClaudeThinkingMode,
                        })
                        await saveCurrentProfile()
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue
                          placeholder={t(
                            'claude.reasoning.thinkingModePlaceholder'
                          )}
                        />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="inherit">
                          {t('claude.reasoning.inherit')}
                        </SelectItem>
                        <SelectItem value="on">
                          {t('claude.reasoning.on')}
                        </SelectItem>
                        <SelectItem value="off">
                          {t('claude.reasoning.off')}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </div>

              <ConfigStatus status={configStatus} />
            </div>
          </>
        )}
      </div>

      <AlertDialog open={showApplyConfirm} onOpenChange={setShowApplyConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('claude.actions.apply')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('claude.actions.applyConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleApply}>
              {t('claude.actions.apply')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={showDeleteProfileConfirm}
        onOpenChange={setShowDeleteProfileConfirm}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('claude.profile.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('claude.profile.deleteConfirm')}
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

      <Dialog
        open={showCreateProfileDialog}
        onOpenChange={setShowCreateProfileDialog}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('claude.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('claude.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="claude-create-profile">
              {t('claude.profile.name')}
            </Label>
            <Input
              id="claude-create-profile"
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('claude.profile.namePlaceholder')}
              onKeyDown={async e => {
                if (e.key === 'Enter') {
                  await handleCreateProfile()
                }
              }}
            />
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowCreateProfileDialog(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button onClick={handleCreateProfile}>
              {t('claude.profile.createAction')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={showDuplicateProfileDialog}
        onOpenChange={setShowDuplicateProfileDialog}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('claude.profile.duplicate')}</DialogTitle>
            <DialogDescription>
              {t('claude.profile.duplicateDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="claude-duplicate-profile">
              {t('claude.profile.name')}
            </Label>
            <Input
              id="claude-duplicate-profile"
              value={newProfileName}
              onChange={e => setNewProfileName(e.target.value)}
              placeholder={t('claude.profile.namePlaceholder')}
              onKeyDown={async e => {
                if (e.key === 'Enter') {
                  await handleDuplicateProfile()
                }
              }}
            />
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowDuplicateProfileDialog(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button onClick={handleDuplicateProfile}>
              {t('claude.profile.duplicateAction')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
