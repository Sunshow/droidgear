import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertCircle, Copy, Check, RefreshCw } from 'lucide-react'
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

const ENV_VAR_NAME = 'FACTORY_API_KEY'
const AUTO_UPDATE_ENV_VAR_NAME = 'FACTORY_DROID_AUTO_UPDATE_ENABLED'

function generateRandomKey(): string {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
  let result = 'fk-'
  for (let i = 0; i < 24; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  return result
}

type ShellType = 'bash' | 'zsh' | 'powershell'

function CopyableCommand({
  command,
  onCopy,
}: {
  command: string
  onCopy: () => void
}) {
  const [copied, setCopied] = useState(false)
  const { t } = useTranslation()

  const handleCopy = async () => {
    await writeText(command)
    setCopied(true)
    onCopy()
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="flex items-center gap-2">
      <code className="flex-1 p-2 bg-muted rounded-md text-sm font-mono overflow-x-auto">
        {command}
      </code>
      <Button
        variant="outline"
        size="icon"
        className="shrink-0"
        onClick={handleCopy}
        title={t('common.copy')}
      >
        {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
      </Button>
    </div>
  )
}

function ShellCommandSection({
  shell,
  apiKeyValue,
}: {
  shell: ShellType
  apiKeyValue: string
}) {
  const { t } = useTranslation()

  const getCommand = () => {
    if (shell === 'powershell') {
      return `$env:${ENV_VAR_NAME} = "${apiKeyValue}"`
    }
    return `export ${ENV_VAR_NAME}="${apiKeyValue}"`
  }

  const labelKey = `droid.helpers.skipLogin.instructions.${shell}`
  const pathKey = `droid.helpers.skipLogin.instructions.${shell}Path`

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-sm">
        <span className="font-medium">{t(labelKey)}</span>
        <span className="text-muted-foreground">- {t(pathKey)}</span>
      </div>
      <CopyableCommand
        command={getCommand()}
        onCopy={() => toast.success(t('common.copied'))}
      />
    </div>
  )
}

function EnvVarShellCommandSection({
  shell,
  envVarName,
  envVarValue,
}: {
  shell: ShellType
  envVarName: string
  envVarValue: string
}) {
  const { t } = useTranslation()

  const getCommand = () => {
    if (shell === 'powershell') {
      return `$env:${envVarName} = "${envVarValue}"`
    }
    return `export ${envVarName}="${envVarValue}"`
  }

  const labelKey = `droid.helpers.skipLogin.instructions.${shell}`
  const pathKey = `droid.helpers.skipLogin.instructions.${shell}Path`

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 text-sm">
        <span className="font-medium">{t(labelKey)}</span>
        <span className="text-muted-foreground">- {t(pathKey)}</span>
      </div>
      <CopyableCommand
        command={getCommand()}
        onCopy={() => toast.success(t('common.copied'))}
      />
    </div>
  )
}

export function DroidHelpersPage() {
  const { t } = useTranslation()

  const [setupDialogOpen, setSetupDialogOpen] = useState(false)
  const [disableAutoUpdateDialogOpen, setDisableAutoUpdateDialogOpen] =
    useState(false)
  const [apiKeyValue, setApiKeyValue] = useState('')
  const [cloudSessionSync, setCloudSessionSync] = useState(true)

  // Session settings states
  const [reasoningEffort, setReasoningEffort] = useState<string | null>(null)
  const [diffMode, setDiffMode] = useState('github')
  const [todoDisplayMode, setTodoDisplayMode] = useState('pinned')
  const [includeCoAuthoredByDroid, setIncludeCoAuthoredByDroid] = useState(true)
  const [showThinkingInMainView, setShowThinkingInMainView] = useState(false)

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

  const handleOpenSetupDialog = () => {
    setApiKeyValue(generateRandomKey())
    setSetupDialogOpen(true)
  }

  const handleCloudSessionSyncToggle = async (enabled: boolean) => {
    setCloudSessionSync(enabled)
    const result = await commands.saveCloudSessionSync(enabled)
    if (result.status === 'error') {
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

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('droid.helpers.title')}</h1>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-6">
          {/* Environment Variable Conflict Warning */}
          <div className="p-4 bg-amber-500/10 border border-amber-500/20 rounded-md space-y-2">
            <div className="flex items-center gap-2 text-amber-600 dark:text-amber-500">
              <AlertCircle className="h-5 w-5 shrink-0" />
              <span className="font-medium">
                {t('droid.helpers.envConflict.title')}
              </span>
            </div>
            <p className="text-sm text-muted-foreground">
              {t('droid.helpers.envConflict.description')}
            </p>
            <p className="text-sm text-muted-foreground">
              {t('droid.helpers.envConflict.solution')}
            </p>
          </div>

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
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-1">
                <Label className="text-base font-medium">
                  {t('droid.helpers.skipLogin.title')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t('droid.helpers.skipLogin.description')}
                </p>
              </div>
              <Button variant="outline" onClick={handleOpenSetupDialog}>
                {t('droid.helpers.skipLogin.setupButton')}
              </Button>
            </div>
            <div className="flex items-center gap-2 text-amber-600 dark:text-amber-500">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <p className="text-xs">
                {t('droid.helpers.skipLogin.toolLimitationHint')}
              </p>
            </div>
          </div>

          {/* Disable Auto Update Section */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-1">
                <Label className="text-base font-medium">
                  {t('droid.helpers.disableAutoUpdate.title')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t('droid.helpers.disableAutoUpdate.description')}
                </p>
              </div>
              <Button
                variant="outline"
                onClick={() => setDisableAutoUpdateDialogOpen(true)}
              >
                {t('droid.helpers.disableAutoUpdate.setupButton')}
              </Button>
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

      {/* Setup Dialog */}
      <Dialog open={setupDialogOpen} onOpenChange={setSetupDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{t('droid.helpers.skipLogin.title')}</DialogTitle>
            <DialogDescription>
              {t('droid.helpers.skipLogin.description')}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('droid.helpers.skipLogin.apiKeyLabel')}</Label>
              <div className="flex gap-2">
                <Input
                  value={apiKeyValue}
                  onChange={e => setApiKeyValue(e.target.value)}
                  placeholder={t('droid.helpers.skipLogin.apiKeyPlaceholder')}
                  className="font-mono flex-1"
                />
                <Button
                  variant="outline"
                  size="icon"
                  onClick={() => setApiKeyValue(generateRandomKey())}
                  title={t('common.refresh')}
                >
                  <RefreshCw className="h-4 w-4" />
                </Button>
              </div>
            </div>

            <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-md">
              <p className="text-sm text-yellow-600 dark:text-yellow-500">
                ⚠️ {t('droid.helpers.skipLogin.warning')}
              </p>
            </div>

            <div className="space-y-4 pt-2">
              <p className="text-sm font-medium">
                {t('droid.helpers.skipLogin.instructions.title')}
              </p>
              <ShellCommandSection shell="zsh" apiKeyValue={apiKeyValue} />
              <ShellCommandSection shell="bash" apiKeyValue={apiKeyValue} />
              <ShellCommandSection
                shell="powershell"
                apiKeyValue={apiKeyValue}
              />
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setSetupDialogOpen(false)}>
              {t('common.dismiss')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Disable Auto Update Dialog */}
      <Dialog
        open={disableAutoUpdateDialogOpen}
        onOpenChange={setDisableAutoUpdateDialogOpen}
      >
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>
              {t('droid.helpers.disableAutoUpdate.title')}
            </DialogTitle>
            <DialogDescription>
              {t('droid.helpers.disableAutoUpdate.description')}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-md">
              <p className="text-sm text-yellow-600 dark:text-yellow-500">
                ⚠️ {t('droid.helpers.disableAutoUpdate.warning')}
              </p>
            </div>

            <div className="space-y-4 pt-2">
              <p className="text-sm font-medium">
                {t('droid.helpers.disableAutoUpdate.instructions.title')}
              </p>
              <EnvVarShellCommandSection
                shell="zsh"
                envVarName={AUTO_UPDATE_ENV_VAR_NAME}
                envVarValue="0"
              />
              <EnvVarShellCommandSection
                shell="bash"
                envVarName={AUTO_UPDATE_ENV_VAR_NAME}
                envVarValue="0"
              />
              <EnvVarShellCommandSection
                shell="powershell"
                envVarName={AUTO_UPDATE_ENV_VAR_NAME}
                envVarValue="0"
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setDisableAutoUpdateDialogOpen(false)}
            >
              {t('common.dismiss')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
