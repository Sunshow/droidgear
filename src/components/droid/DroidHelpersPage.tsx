import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Info, Copy, Check, Pencil, AlertCircle } from 'lucide-react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'sonner'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
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
import { commands } from '@/lib/bindings'
import { usePlatform } from '@/hooks/use-platform'

const ENV_VAR_NAME = 'FACTORY_API_KEY'
const DEFAULT_KEY = 'fk-your-key-here'

export function DroidHelpersPage() {
  const { t } = useTranslation()
  const platform = usePlatform()

  const [envValue, setEnvValue] = useState<string | null>(null)
  const [helpDialogOpen, setHelpDialogOpen] = useState(false)
  const [editDialogOpen, setEditDialogOpen] = useState(false)
  const [editValue, setEditValue] = useState('')
  const [copied, setCopied] = useState(false)
  const [cloudSessionSync, setCloudSessionSync] = useState(true)

  // Session settings states
  const [reasoningEffort, setReasoningEffort] = useState<string | null>(null)
  const [diffMode, setDiffMode] = useState('github')
  const [todoDisplayMode, setTodoDisplayMode] = useState('pinned')
  const [includeCoAuthoredByDroid, setIncludeCoAuthoredByDroid] = useState(true)
  const [showThinkingInMainView, setShowThinkingInMainView] = useState(false)

  const isEnabled = envValue !== null && envValue.length > 0

  useEffect(() => {
    let cancelled = false
    const fetchEnvVar = async () => {
      const value = await commands.getEnvVar(ENV_VAR_NAME)
      if (!cancelled) {
        setEnvValue(value)
      }
    }
    fetchEnvVar()
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false
    const fetchCloudSessionSync = async () => {
      const result = await commands.getCloudSessionSync()
      if (!cancelled && result.status === 'ok') {
        setCloudSessionSync(result.data)
      }
    }
    fetchCloudSessionSync()
    return () => {
      cancelled = true
    }
  }, [])

  // Fetch session settings on mount
  useEffect(() => {
    let cancelled = false
    const fetchSessionSettings = async () => {
      const [
        reasoningEffortResult,
        diffModeResult,
        todoDisplayModeResult,
        includeCoAuthoredResult,
        showThinkingResult,
      ] = await Promise.all([
        commands.getReasoningEffort(),
        commands.getDiffMode(),
        commands.getTodoDisplayMode(),
        commands.getIncludeCoAuthoredByDroid(),
        commands.getShowThinkingInMainView(),
      ])

      if (cancelled) return

      if (reasoningEffortResult.status === 'ok') {
        setReasoningEffort(reasoningEffortResult.data)
      }
      if (diffModeResult.status === 'ok') {
        setDiffMode(diffModeResult.data)
      }
      if (todoDisplayModeResult.status === 'ok') {
        setTodoDisplayMode(todoDisplayModeResult.data)
      }
      if (includeCoAuthoredResult.status === 'ok') {
        setIncludeCoAuthoredByDroid(includeCoAuthoredResult.data)
      }
      if (showThinkingResult.status === 'ok') {
        setShowThinkingInMainView(showThinkingResult.data)
      }
    }
    fetchSessionSettings()
    return () => {
      cancelled = true
    }
  }, [])

  const refreshEnvVar = async () => {
    const value = await commands.getEnvVar(ENV_VAR_NAME)
    setEnvValue(value)
  }

  const handleToggle = async (enabled: boolean) => {
    if (enabled) {
      await commands.setEnvVar(ENV_VAR_NAME, DEFAULT_KEY)
      await refreshEnvVar()
      setHelpDialogOpen(true)
    } else {
      await commands.removeEnvVar(ENV_VAR_NAME)
      await refreshEnvVar()
    }
  }

  const handleEdit = () => {
    setEditValue(envValue ?? DEFAULT_KEY)
    setEditDialogOpen(true)
  }

  const handleSaveEdit = async () => {
    if (editValue.trim()) {
      await commands.setEnvVar(ENV_VAR_NAME, editValue.trim())
      await refreshEnvVar()
      setEditDialogOpen(false)
      toast.success(t('common.save'))
    }
  }

  const handleCopy = async () => {
    const keyValue = envValue ?? DEFAULT_KEY
    const command =
      platform === 'windows'
        ? `set ${ENV_VAR_NAME}=${keyValue}`
        : `export ${ENV_VAR_NAME}="${keyValue}"`

    await writeText(command)
    setCopied(true)
    toast.success(t('common.copied'))
    setTimeout(() => setCopied(false), 2000)
  }

  const handleCloudSessionSyncToggle = async (enabled: boolean) => {
    setCloudSessionSync(enabled)
    const result = await commands.saveCloudSessionSync(enabled)
    if (result.status === 'error') {
      // Revert on error
      setCloudSessionSync(!enabled)
      toast.error(t('toast.error.generic'))
    }
  }

  const handleReasoningEffortChange = async (value: string) => {
    const oldValue = reasoningEffort
    setReasoningEffort(value)
    const result = await commands.saveReasoningEffort(value)
    if (result.status === 'error') {
      setReasoningEffort(oldValue)
      toast.error(t('toast.error.generic'))
    }
  }

  const handleDiffModeChange = async (value: string) => {
    const oldValue = diffMode
    setDiffMode(value)
    const result = await commands.saveDiffMode(value)
    if (result.status === 'error') {
      setDiffMode(oldValue)
      toast.error(t('toast.error.generic'))
    }
  }

  const handleTodoDisplayModeChange = async (value: string) => {
    const oldValue = todoDisplayMode
    setTodoDisplayMode(value)
    const result = await commands.saveTodoDisplayMode(value)
    if (result.status === 'error') {
      setTodoDisplayMode(oldValue)
      toast.error(t('toast.error.generic'))
    }
  }

  const handleIncludeCoAuthoredByDroidChange = async (enabled: boolean) => {
    const oldValue = includeCoAuthoredByDroid
    setIncludeCoAuthoredByDroid(enabled)
    const result = await commands.saveIncludeCoAuthoredByDroid(enabled)
    if (result.status === 'error') {
      setIncludeCoAuthoredByDroid(oldValue)
      toast.error(t('toast.error.generic'))
    }
  }

  const handleShowThinkingInMainViewChange = async (enabled: boolean) => {
    const oldValue = showThinkingInMainView
    setShowThinkingInMainView(enabled)
    const result = await commands.saveShowThinkingInMainView(enabled)
    if (result.status === 'error') {
      setShowThinkingInMainView(oldValue)
      toast.error(t('toast.error.generic'))
    }
  }

  const maskedValue = envValue
    ? envValue.length > 10
      ? `${envValue.substring(0, 6)}${'*'.repeat(Math.min(envValue.length - 10, 16))}${envValue.substring(envValue.length - 4)}`
      : envValue
    : null

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('droid.helpers.title')}</h1>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-6">
          {/* Cloud Session Sync Section */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-1">
                <Label
                  htmlFor="cloud-session-sync"
                  className="text-base font-medium"
                >
                  {t('droid.helpers.cloudSessionSync.title')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t('droid.helpers.cloudSessionSync.description')}
                </p>
              </div>
              <Switch
                id="cloud-session-sync"
                checked={cloudSessionSync}
                onCheckedChange={handleCloudSessionSyncToggle}
              />
            </div>
          </div>

          {/* Skip Login Section */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Label htmlFor="skip-login" className="text-base font-medium">
                  {t('droid.helpers.skipLogin.title')}
                </Label>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() => setHelpDialogOpen(true)}
                >
                  <Info className="h-4 w-4 text-muted-foreground" />
                </Button>
              </div>
              <Switch
                id="skip-login"
                checked={isEnabled}
                onCheckedChange={handleToggle}
              />
            </div>

            {/* Environment Variable Display */}
            <div className="space-y-2">
              <Label className="text-sm text-muted-foreground">
                {ENV_VAR_NAME}
              </Label>
              <div className="flex items-center gap-2">
                <div className="flex-1 p-3 bg-muted rounded-md font-mono text-sm">
                  {maskedValue ?? (
                    <span className="text-muted-foreground">
                      {t('droid.helpers.skipLogin.notSet')}
                    </span>
                  )}
                </div>
                {isEnabled && (
                  <Button
                    variant="outline"
                    size="icon"
                    className="shrink-0"
                    onClick={handleEdit}
                  >
                    <Pencil className="h-4 w-4" />
                  </Button>
                )}
              </div>
            </div>
          </div>

          {/* Session Settings Section */}
          <div className="space-y-4 pt-4 border-t">
            <h2 className="text-base font-medium">
              {t('droid.helpers.sessionSettings.title')}
            </h2>

            {/* Reasoning Effort */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label
                    htmlFor="reasoning-effort"
                    className="text-sm font-medium"
                  >
                    {t('droid.helpers.sessionSettings.reasoningEffort')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t(
                      'droid.helpers.sessionSettings.reasoningEffortDescription'
                    )}
                  </p>
                </div>
                <Select
                  value={reasoningEffort ?? 'off'}
                  onValueChange={handleReasoningEffortChange}
                >
                  <SelectTrigger className="w-32">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="off">
                      {t('droid.helpers.sessionSettings.reasoningEffort.off')}
                    </SelectItem>
                    <SelectItem value="low">
                      {t('droid.helpers.sessionSettings.reasoningEffort.low')}
                    </SelectItem>
                    <SelectItem value="medium">
                      {t(
                        'droid.helpers.sessionSettings.reasoningEffort.medium'
                      )}
                    </SelectItem>
                    <SelectItem value="high">
                      {t('droid.helpers.sessionSettings.reasoningEffort.high')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center gap-2 text-amber-600 dark:text-amber-500">
                <AlertCircle className="h-4 w-4 shrink-0" />
                <p className="text-xs">
                  {t('droid.helpers.sessionSettings.reasoningEffortNote')}
                </p>
              </div>
            </div>

            {/* Diff Mode */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label htmlFor="diff-mode" className="text-sm font-medium">
                    {t('droid.helpers.sessionSettings.diffMode')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t('droid.helpers.sessionSettings.diffModeDescription')}
                  </p>
                </div>
                <Select value={diffMode} onValueChange={handleDiffModeChange}>
                  <SelectTrigger className="w-40">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="github">
                      {t('droid.helpers.sessionSettings.diffMode.github')}
                    </SelectItem>
                    <SelectItem value="unified">
                      {t('droid.helpers.sessionSettings.diffMode.unified')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Todo Display Mode */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label
                    htmlFor="todo-display-mode"
                    className="text-sm font-medium"
                  >
                    {t('droid.helpers.sessionSettings.todoDisplayMode')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t(
                      'droid.helpers.sessionSettings.todoDisplayModeDescription'
                    )}
                  </p>
                </div>
                <Select
                  value={todoDisplayMode}
                  onValueChange={handleTodoDisplayModeChange}
                >
                  <SelectTrigger className="w-32">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="pinned">
                      {t(
                        'droid.helpers.sessionSettings.todoDisplayMode.pinned'
                      )}
                    </SelectItem>
                    <SelectItem value="inline">
                      {t(
                        'droid.helpers.sessionSettings.todoDisplayMode.inline'
                      )}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Include Co-Authored By Droid */}
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-1">
                <Label
                  htmlFor="include-co-authored"
                  className="text-sm font-medium"
                >
                  {t('droid.helpers.sessionSettings.includeCoAuthoredByDroid')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t(
                    'droid.helpers.sessionSettings.includeCoAuthoredByDroidDescription'
                  )}
                </p>
              </div>
              <Switch
                id="include-co-authored"
                checked={includeCoAuthoredByDroid}
                onCheckedChange={handleIncludeCoAuthoredByDroidChange}
              />
            </div>

            {/* Show Thinking In Main View */}
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-1">
                <Label htmlFor="show-thinking" className="text-sm font-medium">
                  {t('droid.helpers.sessionSettings.showThinkingInMainView')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t(
                    'droid.helpers.sessionSettings.showThinkingInMainViewDescription'
                  )}
                </p>
              </div>
              <Switch
                id="show-thinking"
                checked={showThinkingInMainView}
                onCheckedChange={handleShowThinkingInMainViewChange}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Edit Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t('droid.helpers.skipLogin.editTitle')}</DialogTitle>
            <DialogDescription>
              {t('droid.helpers.skipLogin.editDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={editValue}
              onChange={e => setEditValue(e.target.value)}
              placeholder={DEFAULT_KEY}
              className="font-mono"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={handleSaveEdit}>{t('common.save')}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Help Dialog */}
      <Dialog open={helpDialogOpen} onOpenChange={setHelpDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{t('droid.helpers.skipLogin.helpTitle')}</DialogTitle>
            <DialogDescription>
              {t('droid.helpers.skipLogin.helpDescription')}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            {/* Copy Command */}
            <div className="space-y-2">
              <Label>{t('droid.helpers.skipLogin.copyCommand')}</Label>
              <div className="flex items-center gap-2">
                <code className="flex-1 p-2 bg-muted rounded-md text-sm font-mono overflow-x-auto">
                  {platform === 'windows'
                    ? `set ${ENV_VAR_NAME}=${envValue ?? DEFAULT_KEY}`
                    : `export ${ENV_VAR_NAME}="${envValue ?? DEFAULT_KEY}"`}
                </code>
                <Button
                  variant="outline"
                  size="icon"
                  className="shrink-0"
                  onClick={handleCopy}
                >
                  {copied ? (
                    <Check className="h-4 w-4" />
                  ) : (
                    <Copy className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </div>

            {/* Platform-specific instructions */}
            <div className="space-y-3">
              {platform === 'windows' ? (
                <>
                  <div>
                    <h4 className="font-medium mb-1">CMD:</h4>
                    <code className="block p-2 bg-muted rounded-md text-sm font-mono">
                      {`set ${ENV_VAR_NAME}=${envValue ?? DEFAULT_KEY}`}
                    </code>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">PowerShell:</h4>
                    <code className="block p-2 bg-muted rounded-md text-sm font-mono">
                      {`$env:${ENV_VAR_NAME}="${envValue ?? DEFAULT_KEY}"`}
                    </code>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">
                      {t('droid.helpers.skipLogin.permanent')}:
                    </h4>
                    <p className="text-sm text-muted-foreground">
                      {t('droid.helpers.skipLogin.windowsPermanent')}
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <div>
                    <h4 className="font-medium mb-1">
                      {t('droid.helpers.skipLogin.temporary')}:
                    </h4>
                    <code className="block p-2 bg-muted rounded-md text-sm font-mono">
                      {`export ${ENV_VAR_NAME}="${envValue ?? DEFAULT_KEY}"`}
                    </code>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">
                      {t('droid.helpers.skipLogin.permanent')}:
                    </h4>
                    <p className="text-sm text-muted-foreground">
                      {platform === 'macos'
                        ? t('droid.helpers.skipLogin.macosPermanent')
                        : t('droid.helpers.skipLogin.linuxPermanent')}
                    </p>
                  </div>
                </>
              )}
            </div>

            {/* Warning */}
            <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-md">
              <p className="text-sm text-yellow-600 dark:text-yellow-500">
                ⚠️ {t('droid.helpers.skipLogin.restartWarning')}
              </p>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
