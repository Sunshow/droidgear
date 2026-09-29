import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  AlertCircle,
  Copy,
  Download,
  Play,
  Plus,
  RefreshCw,
  Trash2,
} from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { toast } from 'sonner'
import { commands, type CopilotProfile } from '@/lib/bindings'
import { useCopilotStore } from '@/store/copilot-store'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { ChannelModelPickerDialog } from '@/components/channels/ChannelModelPickerDialog'
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

function positiveNumber(value: string): number | null {
  const number = Number(value.trim())
  return Number.isSafeInteger(number) && number > 0 ? number : null
}

export function CopilotConfigPage() {
  const { t } = useTranslation()
  const profiles = useCopilotStore(state => state.profiles)
  const activeProfileId = useCopilotStore(state => state.activeProfileId)
  const currentProfile = useCopilotStore(state => state.currentProfile)
  const isLoading = useCopilotStore(state => state.isLoading)
  const error = useCopilotStore(state => state.error)
  const configStatus = useCopilotStore(state => state.configStatus)
  const loadProfiles = useCopilotStore(state => state.loadProfiles)
  const loadActiveProfileId = useCopilotStore(
    state => state.loadActiveProfileId
  )
  const loadConfigStatus = useCopilotStore(state => state.loadConfigStatus)
  const selectProfile = useCopilotStore(state => state.selectProfile)
  const createProfile = useCopilotStore(state => state.createProfile)
  const saveProfile = useCopilotStore(state => state.saveProfile)
  const deleteProfile = useCopilotStore(state => state.deleteProfile)
  const duplicateProfile = useCopilotStore(state => state.duplicateProfile)
  const applyProfile = useCopilotStore(state => state.applyProfile)
  const loadFromLiveConfig = useCopilotStore(state => state.loadFromLiveConfig)
  const importFromChannel = useCopilotStore(state => state.importFromChannel)
  const setError = useCopilotStore(state => state.setError)

  const [newProfileName, setNewProfileName] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [showDuplicate, setShowDuplicate] = useState(false)
  const [showDelete, setShowDelete] = useState(false)
  const [showApply, setShowApply] = useState(false)
  const [isLaunching, setIsLaunching] = useState(false)
  const [showChannelPicker, setShowChannelPicker] = useState(false)

  useEffect(() => {
    const load = async () => {
      await loadProfiles()
      await loadActiveProfileId()
    }
    load()
    loadConfigStatus()
  }, [loadProfiles, loadActiveProfileId, loadConfigStatus])

  const updateProfile = (updates: Partial<CopilotProfile>) => {
    const profile = useCopilotStore.getState().currentProfile
    if (!profile) return
    useCopilotStore.setState({
      currentProfile: {
        ...profile,
        ...updates,
        updatedAt: new Date().toISOString(),
      },
    })
  }

  const handleCreate = async () => {
    const name = newProfileName.trim()
    if (!name) return
    try {
      await createProfile(name)
      if (useCopilotStore.getState().error) return
      setNewProfileName('')
      setShowCreate(false)
    } catch (createError) {
      toast.error(String(createError))
    }
  }

  const handleDuplicate = async () => {
    const profile = currentProfile
    const name = newProfileName.trim()
    if (!profile || !name) return
    await duplicateProfile(profile.id, name)
    if (useCopilotStore.getState().error) return
    setNewProfileName('')
    setShowDuplicate(false)
  }

  const handleSave = async () => {
    await saveProfile()
    const state = useCopilotStore.getState()
    if (state.error) toast.error(state.error)
    else toast.success(t('copilot.actions.saveSuccess'))
  }

  const handleApply = async () => {
    if (!currentProfile) return
    await applyProfile(currentProfile.id)
    const state = useCopilotStore.getState()
    if (state.error) toast.error(state.error)
    else toast.success(t('copilot.actions.applySuccess'))
    setShowApply(false)
  }

  const handleLaunch = async () => {
    if (!currentProfile || isLaunching) return
    setIsLaunching(true)
    setError(null)
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t('copilot.actions.selectDirectory'),
      })
      if (typeof selected !== 'string') return
      await saveProfile()
      const saved = useCopilotStore.getState()
      if (saved.error) {
        toast.error(saved.error)
        return
      }
      const result = await commands.launchCopilot(currentProfile.id, selected)
      if (result.status === 'ok')
        toast.success(t('copilot.actions.launchSuccess'))
      else {
        setError(result.error)
        toast.error(result.error)
      }
    } catch (launchError) {
      setError(String(launchError))
      toast.error(String(launchError))
    } finally {
      setIsLaunching(false)
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between gap-2 border-b p-4">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('copilot.title')}</h1>
          <div className="mt-1 flex items-center gap-2">
            {currentProfile && activeProfileId === currentProfile.id && (
              <Badge variant="outline">{t('copilot.profile.active')}</Badge>
            )}
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          <Button
            onClick={handleLaunch}
            disabled={!currentProfile || isLoading || isLaunching}
            title={t('copilot.actions.launchTooltip')}
          >
            <Play className="mr-2 h-4 w-4" />
            {t('copilot.actions.launch')}
          </Button>
          <Button
            variant="outline"
            size="icon"
            onClick={async () => {
              await loadProfiles()
              await loadActiveProfileId()
              await loadConfigStatus()
            }}
            disabled={isLoading}
            title={t('common.refresh')}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            onClick={() => setShowApply(true)}
            disabled={!currentProfile || isLoading || isLaunching}
          >
            {t('copilot.actions.apply')}
          </Button>
          <Button onClick={handleSave} disabled={!currentProfile || isLoading}>
            {t('copilot.actions.save')}
          </Button>
        </div>
      </div>

      {error && (
        <div className="mx-4 mt-4 flex items-center gap-2 rounded-md border border-destructive/20 bg-destructive/10 p-3 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          <span>{error}</span>
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

      <div className="flex-1 space-y-4 overflow-auto p-4">
        <section className="space-y-3 rounded-lg border p-4">
          <div className="flex items-center gap-2">
            <Label className="w-24 shrink-0">
              {t('copilot.profile.select')}
            </Label>
            <Select
              value={currentProfile?.id ?? ''}
              onValueChange={selectProfile}
              disabled={isLoading || isLaunching}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t('copilot.profile.select')} />
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
              onClick={() => setShowCreate(true)}
              title={t('copilot.profile.create')}
            >
              <Plus className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => {
                setNewProfileName(
                  currentProfile ? `${currentProfile.name} (Copy)` : ''
                )
                setShowDuplicate(true)
              }}
              disabled={!currentProfile}
              title={t('copilot.profile.duplicate')}
            >
              <Copy className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setShowDelete(true)}
              disabled={!currentProfile || profiles.length <= 1}
              title={t('copilot.profile.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>

          {currentProfile && (
            <>
              <div className="flex items-center gap-2">
                <Label className="w-24 shrink-0">
                  {t('copilot.profile.name')}
                </Label>
                <Input
                  className="flex-1"
                  value={currentProfile.name}
                  onChange={event =>
                    updateProfile({ name: event.target.value })
                  }
                  placeholder={t('copilot.profile.namePlaceholder')}
                />
              </div>
              <div className="flex items-center gap-2">
                <Label className="w-24 shrink-0">
                  {t('copilot.profile.description')}
                </Label>
                <Input
                  className="flex-1"
                  value={currentProfile.description ?? ''}
                  onChange={event =>
                    updateProfile({ description: event.target.value || null })
                  }
                  placeholder={t('copilot.profile.descriptionPlaceholder')}
                />
              </div>
            </>
          )}
        </section>

        {currentProfile && (
          <section className="space-y-3 rounded-lg border p-4">
            <div>
              <h2 className="text-lg font-medium">
                {t('copilot.provider.title')}
              </h2>
              <p className="text-xs text-muted-foreground">
                {t('copilot.provider.hint')}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Label className="w-24 shrink-0">
                {t('copilot.provider.mode')}
              </Label>
              <Select
                value={currentProfile.useOfficialAuth ? 'official' : 'byok'}
                onValueChange={value =>
                  updateProfile({ useOfficialAuth: value === 'official' })
                }
              >
                <SelectTrigger className="flex-1">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="official">
                    {t('copilot.provider.officialOption')}
                  </SelectItem>
                  <SelectItem value="byok">
                    {t('copilot.provider.byokOption')}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            {!currentProfile.useOfficialAuth && (
              <div className="space-y-3 rounded-md border p-3">
                <div className="flex items-center gap-2">
                  <Label className="w-36 shrink-0">
                    {t('copilot.provider.baseUrl')}
                  </Label>
                  <Input
                    className="flex-1"
                    value={currentProfile.baseUrl ?? ''}
                    onChange={event =>
                      updateProfile({ baseUrl: event.target.value })
                    }
                    placeholder={t('copilot.provider.baseUrlPlaceholder')}
                    aria-describedby="copilot-base-url-hint"
                  />
                </div>
                <p
                  id="copilot-base-url-hint"
                  className="text-xs text-muted-foreground"
                >
                  {t('copilot.provider.baseUrlHint')}
                </p>
                <div className="flex items-center gap-2">
                  <Label className="w-36 shrink-0">
                    {t('copilot.provider.providerType')}
                  </Label>
                  <Select
                    value={currentProfile.providerType || 'openai'}
                    onValueChange={providerType =>
                      updateProfile({ providerType })
                    }
                  >
                    <SelectTrigger className="flex-1">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {['openai', 'anthropic'].map(type => (
                        <SelectItem key={type} value={type}>
                          {type}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-center gap-2">
                  <Label className="w-36 shrink-0">
                    {t('copilot.provider.apiKey')}
                  </Label>
                  <Input
                    className="flex-1"
                    type="password"
                    value={currentProfile.apiKey ?? ''}
                    onChange={event =>
                      updateProfile({ apiKey: event.target.value })
                    }
                    placeholder={t('copilot.provider.apiKeyPlaceholder')}
                  />
                </div>
                <div className="flex items-center gap-2">
                  <Label className="w-36 shrink-0">
                    {t('copilot.provider.model')}
                  </Label>
                  <Input
                    className="flex-1"
                    value={currentProfile.model ?? ''}
                    onChange={event =>
                      updateProfile({ model: event.target.value })
                    }
                    placeholder={t('copilot.provider.modelPlaceholder')}
                  />
                </div>
                <div className="flex items-center gap-2">
                  <Label className="w-36 shrink-0">
                    {t('copilot.provider.maxPromptTokens')}
                  </Label>
                  <Input
                    className="flex-1"
                    inputMode="numeric"
                    value={currentProfile.maxPromptTokens?.toString() ?? ''}
                    onChange={event =>
                      updateProfile({
                        maxPromptTokens: positiveNumber(event.target.value),
                      })
                    }
                    placeholder={t(
                      'copilot.provider.maxPromptTokensPlaceholder'
                    )}
                  />
                </div>
                <div className="flex items-center gap-2">
                  <Label className="w-36 shrink-0">
                    {t('copilot.provider.maxOutputTokens')}
                  </Label>
                  <Input
                    className="flex-1"
                    inputMode="numeric"
                    value={currentProfile.maxOutputTokens?.toString() ?? ''}
                    onChange={event =>
                      updateProfile({
                        maxOutputTokens: positiveNumber(event.target.value),
                      })
                    }
                    placeholder={t(
                      'copilot.provider.maxOutputTokensPlaceholder'
                    )}
                  />
                </div>
              </div>
            )}
            <Button
              variant="outline"
              onClick={() => setShowChannelPicker(true)}
              disabled={isLoading || isLaunching}
            >
              <Download className="mr-2 h-4 w-4" />
              {t('channels.importFromChannel')}
            </Button>
            <p className="text-xs text-muted-foreground">
              {t('copilot.provider.importHint')}
            </p>
            <Button
              variant="outline"
              onClick={loadFromLiveConfig}
              disabled={!configStatus?.configExists}
            >
              {t('copilot.provider.loadFromConfig')}
            </Button>
          </section>
        )}

        {configStatus && (
          <section className="space-y-2 rounded-lg border p-4">
            <h2 className="text-sm font-medium text-muted-foreground">
              {t('copilot.configStatus.title')}
            </h2>
            <div className="flex items-center gap-2 text-sm">
              <code className="flex-1 truncate rounded bg-muted px-1 py-0.5 text-xs select-all">
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
          </section>
        )}
      </div>

      <ChannelModelPickerDialog
        open={showChannelPicker}
        onOpenChange={setShowChannelPicker}
        mode="single"
        onSelect={() => {
          // The context callback supplies the original channel URL and protocol.
        }}
        onSelectWithContext={(models, context) => {
          const model = models[0]
          if (!model) return
          void importFromChannel({
            channelType: context.channelType,
            baseUrl: context.baseUrl,
            apiKey: context.apiKey,
            platform: context.platform,
            provider: context.provider ?? model.provider,
            model: model.model,
            maxOutputTokens: model.maxOutputTokens ?? null,
          })
        }}
        platformFilter={platform =>
          !platform ||
          ['openai', 'anthropic', 'claude', 'grok', 'deepseek'].includes(
            platform.toLowerCase()
          )
        }
      />

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent onCloseAutoFocus={event => event.preventDefault()}>
          <DialogHeader>
            <DialogTitle>{t('copilot.profile.create')}</DialogTitle>
            <DialogDescription>
              {t('copilot.profile.createDescription')}
            </DialogDescription>
          </DialogHeader>
          <Input
            value={newProfileName}
            onChange={event => setNewProfileName(event.target.value)}
            placeholder={t('copilot.profile.namePlaceholder')}
            onKeyDown={event => {
              if (event.key === 'Enter' && !event.nativeEvent.isComposing) {
                void handleCreate()
              }
            }}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={handleCreate} disabled={!newProfileName.trim()}>
              {t('common.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={showDuplicate} onOpenChange={setShowDuplicate}>
        <DialogContent onCloseAutoFocus={event => event.preventDefault()}>
          <DialogHeader>
            <DialogTitle>{t('copilot.profile.duplicate')}</DialogTitle>
            <DialogDescription>
              {t('copilot.profile.duplicateDescription')}
            </DialogDescription>
          </DialogHeader>
          <Input
            value={newProfileName}
            onChange={event => setNewProfileName(event.target.value)}
            placeholder={t('copilot.profile.namePlaceholder')}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowDuplicate(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={handleDuplicate} disabled={!newProfileName.trim()}>
              {t('copilot.profile.duplicate')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={showDelete} onOpenChange={setShowDelete}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('copilot.profile.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('copilot.profile.deleteConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => currentProfile && deleteProfile(currentProfile.id)}
            >
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={showApply} onOpenChange={setShowApply}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('copilot.actions.apply')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('copilot.actions.applyConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleApply}>
              {t('copilot.actions.apply')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
