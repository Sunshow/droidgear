import { useState, useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertCircle, Copy, Check, Plus, Trash2, RefreshCw } from 'lucide-react'
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
import { useUIStore } from '@/store/ui-store'

const AUTO_UPDATE_ENV_VAR_NAME = 'FACTORY_DROID_AUTO_UPDATE_ENABLED'

type ShellType = 'bash' | 'zsh' | 'powershell'

const TOKEN_LIMIT_OPTIONS = [
  { value: 100_000, label: '100K' },
  { value: 200_000, label: '200K', tag: 'default' },
  { value: 250_000, label: '250K', tag: 'recommended' },
  { value: 300_000, label: '300K' },
  { value: 400_000, label: '400K' },
  { value: 500_000, label: '500K' },
  { value: 600_000, label: '600K' },
  { value: 700_000, label: '700K' },
  { value: 800_000, label: '800K' },
  { value: 900_000, label: '900K' },
  { value: 1_000_000, label: '1M' },
]

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

  const labelKey = `droid.settings.shellInstructions.${shell}`
  const pathKey = `droid.settings.shellInstructions.${shell}Path`

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

interface PerModelOverride {
  modelId: string
  tokenLimit: number
}

function TokenLimitSelect({
  value,
  onValueChange,
  className,
}: {
  value: number
  onValueChange: (v: number) => void
  className?: string
}) {
  const { t } = useTranslation()

  return (
    <Select value={String(value)} onValueChange={v => onValueChange(Number(v))}>
      <SelectTrigger className={className ?? 'w-48'}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {TOKEN_LIMIT_OPTIONS.map(opt => (
          <SelectItem key={opt.value} value={String(opt.value)}>
            {opt.label}
            {opt.tag === 'default' &&
              ` (${t('droid.settings.compaction.tokenLimit.defaultTag')})`}
            {opt.tag === 'recommended' &&
              ` (${t('droid.settings.compaction.tokenLimit.recommendedTag')})`}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}

export function DroidSettingsPage() {
  const { t } = useTranslation()

  const disableAutoUpdateRef = useRef<HTMLDivElement>(null)
  const droidSettingsScrollTarget = useUIStore(
    state => state.droidSettingsScrollTarget
  )
  const setDroidSettingsScrollTarget = useUIStore(
    state => state.setDroidSettingsScrollTarget
  )

  useEffect(() => {
    if (droidSettingsScrollTarget === 'disable-auto-update') {
      setTimeout(() => {
        disableAutoUpdateRef.current?.scrollIntoView({ behavior: 'smooth' })
      }, 100)
      setDroidSettingsScrollTarget(null)
    }
  }, [droidSettingsScrollTarget, setDroidSettingsScrollTarget])

  const [disableAutoUpdateDialogOpen, setDisableAutoUpdateDialogOpen] =
    useState(false)
  const [cloudSessionSync, setCloudSessionSync] = useState(true)

  // Session settings states
  const [reasoningEffort, setReasoningEffort] = useState<string | null>(null)
  const [diffMode, setDiffMode] = useState('github')
  const [todoDisplayMode, setTodoDisplayMode] = useState('pinned')
  const [includeCoAuthoredByDroid, setIncludeCoAuthoredByDroid] = useState(true)
  const [showThinkingInMainView, setShowThinkingInMainView] = useState(false)

  // Compaction settings states
  const [compactionModelMode, setCompactionModelMode] =
    useState('current-model')
  const [compactionTokenLimit, setCompactionTokenLimit] = useState(200_000)
  const [perModelOverrides, setPerModelOverrides] = useState<
    PerModelOverride[]
  >([])

  useEffect(() => {
    let cancelled = false
    const fetch = async () => {
      const result = await commands.getCloudSessionSync()
      if (!cancelled && result.status === 'ok') {
        setCloudSessionSync(result.data)
      }
    }
    fetch()
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false
    const fetch = async () => {
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
    fetch()
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false
    const fetch = async () => {
      const [modelModeResult, tokenLimitResult, perModelResult] =
        await Promise.all([
          commands.getCompactionModelMode(),
          commands.getCompactionTokenLimit(),
          commands.getCompactionTokenLimitPerModel(),
        ])
      if (cancelled) return
      if (modelModeResult.status === 'ok') {
        setCompactionModelMode(modelModeResult.data)
      }
      if (tokenLimitResult.status === 'ok') {
        setCompactionTokenLimit(tokenLimitResult.data)
      }
      if (perModelResult.status === 'ok') {
        const entries: PerModelOverride[] = Object.entries(
          perModelResult.data
        ).map(([modelId, tokenLimit]) => ({
          modelId,
          tokenLimit: tokenLimit ?? 200_000,
        }))
        setPerModelOverrides(entries)
      }
    }
    fetch()
    return () => {
      cancelled = true
    }
  }, [])

  const refreshAllSettings = async () => {
    const [
      cloudSyncResult,
      reasoningEffortResult,
      diffModeResult,
      todoDisplayModeResult,
      includeCoAuthoredResult,
      showThinkingResult,
      modelModeResult,
      tokenLimitResult,
      perModelResult,
    ] = await Promise.all([
      commands.getCloudSessionSync(),
      commands.getReasoningEffort(),
      commands.getDiffMode(),
      commands.getTodoDisplayMode(),
      commands.getIncludeCoAuthoredByDroid(),
      commands.getShowThinkingInMainView(),
      commands.getCompactionModelMode(),
      commands.getCompactionTokenLimit(),
      commands.getCompactionTokenLimitPerModel(),
    ])

    if (cloudSyncResult.status === 'ok') {
      setCloudSessionSync(cloudSyncResult.data)
    }
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
    if (modelModeResult.status === 'ok') {
      setCompactionModelMode(modelModeResult.data)
    }
    if (tokenLimitResult.status === 'ok') {
      setCompactionTokenLimit(tokenLimitResult.data)
    }
    if (perModelResult.status === 'ok') {
      const entries: PerModelOverride[] = Object.entries(
        perModelResult.data
      ).map(([modelId, tokenLimit]) => ({
        modelId,
        tokenLimit: tokenLimit ?? 200_000,
      }))
      setPerModelOverrides(entries)
    }
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

  const handleCompactionModelModeChange = async (value: string) => {
    const oldValue = compactionModelMode
    setCompactionModelMode(value)
    const result = await commands.saveCompactionModelMode(value)
    if (result.status === 'error') {
      setCompactionModelMode(oldValue)
      toast.error(t('toast.error.generic'))
    }
  }

  const handleCompactionTokenLimitChange = async (value: number) => {
    const oldValue = compactionTokenLimit
    setCompactionTokenLimit(value)
    const result = await commands.saveCompactionTokenLimit(value)
    if (result.status === 'error') {
      setCompactionTokenLimit(oldValue)
      toast.error(t('toast.error.generic'))
    }
  }

  const savePerModelOverrides = async (overrides: PerModelOverride[]) => {
    const map: Record<string, number> = {}
    for (const o of overrides) {
      if (o.modelId.trim()) {
        map[o.modelId.trim()] = o.tokenLimit
      }
    }
    const result = await commands.saveCompactionTokenLimitPerModel(map)
    if (result.status === 'error') {
      toast.error(t('toast.error.generic'))
    }
  }

  const handleAddPerModelOverride = () => {
    setPerModelOverrides(prev => [
      ...prev,
      { modelId: '', tokenLimit: 200_000 },
    ])
  }

  const handleRemovePerModelOverride = async (index: number) => {
    const updated = perModelOverrides.filter((_, i) => i !== index)
    setPerModelOverrides(updated)
    await savePerModelOverrides(updated)
  }

  const handlePerModelOverrideChange = async (
    index: number,
    field: 'modelId' | 'tokenLimit',
    value: string | number
  ) => {
    const updated = perModelOverrides.map((o, i) =>
      i === index ? { ...o, [field]: value } : o
    )
    setPerModelOverrides(updated)

    // Only save when modelId is not empty
    if (field === 'tokenLimit' || (field === 'modelId' && value)) {
      await savePerModelOverrides(updated)
    }
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('droid.settings.title')}</h1>
        <Button
          variant="ghost"
          size="icon"
          onClick={refreshAllSettings}
          title={t('common.refresh')}
        >
          <RefreshCw className="h-4 w-4" />
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-6">
          {/* Environment Variable Conflict Warning */}
          <div className="p-4 bg-amber-500/10 border border-amber-500/20 rounded-md space-y-2">
            <div className="flex items-center gap-2 text-amber-600 dark:text-amber-500">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span className="font-medium">
                {t('droid.settings.envConflict.title')}
              </span>
            </div>
            <p className="text-sm text-muted-foreground">
              {t('droid.settings.envConflict.description')}
            </p>
            <p className="text-sm text-muted-foreground">
              {t('droid.settings.envConflict.solution')}
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
                  {t('droid.settings.cloudSessionSync.title')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t('droid.settings.cloudSessionSync.description')}
                </p>
              </div>
              <Switch
                id="cloud-session-sync"
                checked={cloudSessionSync}
                onCheckedChange={handleCloudSessionSyncToggle}
              />
            </div>
          </div>

          {/* Disable Auto Update Section */}
          <div
            ref={disableAutoUpdateRef}
            id="disable-auto-update"
            className="space-y-2"
          >
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-1">
                <Label className="text-base font-medium">
                  {t('droid.settings.disableAutoUpdate.title')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t('droid.settings.disableAutoUpdate.description')}
                </p>
              </div>
              <Button
                variant="outline"
                onClick={() => setDisableAutoUpdateDialogOpen(true)}
              >
                {t('droid.settings.disableAutoUpdate.setupButton')}
              </Button>
            </div>
          </div>

          {/* Session Settings Section */}
          <div className="space-y-4 pt-4 border-t">
            <h2 className="text-base font-medium">
              {t('droid.settings.sessionSettings.title')}
            </h2>

            {/* Reasoning Effort */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label
                    htmlFor="reasoning-effort"
                    className="text-sm font-medium"
                  >
                    {t('droid.settings.sessionSettings.reasoningEffort')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t(
                      'droid.settings.sessionSettings.reasoningEffortDescription'
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
                      {t('droid.settings.sessionSettings.reasoningEffort.off')}
                    </SelectItem>
                    <SelectItem value="low">
                      {t('droid.settings.sessionSettings.reasoningEffort.low')}
                    </SelectItem>
                    <SelectItem value="medium">
                      {t(
                        'droid.settings.sessionSettings.reasoningEffort.medium'
                      )}
                    </SelectItem>
                    <SelectItem value="high">
                      {t('droid.settings.sessionSettings.reasoningEffort.high')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center gap-2 text-amber-600 dark:text-amber-500">
                <AlertCircle className="h-4 w-4 shrink-0" />
                <p className="text-xs">
                  {t('droid.settings.sessionSettings.reasoningEffortNote')}
                </p>
              </div>
            </div>

            {/* Diff Mode */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label htmlFor="diff-mode" className="text-sm font-medium">
                    {t('droid.settings.sessionSettings.diffMode')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t('droid.settings.sessionSettings.diffModeDescription')}
                  </p>
                </div>
                <Select value={diffMode} onValueChange={handleDiffModeChange}>
                  <SelectTrigger className="w-40">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="github">
                      {t('droid.settings.sessionSettings.diffMode.github')}
                    </SelectItem>
                    <SelectItem value="unified">
                      {t('droid.settings.sessionSettings.diffMode.unified')}
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
                    {t('droid.settings.sessionSettings.todoDisplayMode')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t(
                      'droid.settings.sessionSettings.todoDisplayModeDescription'
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
                        'droid.settings.sessionSettings.todoDisplayMode.pinned'
                      )}
                    </SelectItem>
                    <SelectItem value="inline">
                      {t(
                        'droid.settings.sessionSettings.todoDisplayMode.inline'
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
                  {t('droid.settings.sessionSettings.includeCoAuthoredByDroid')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t(
                    'droid.settings.sessionSettings.includeCoAuthoredByDroidDescription'
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
                  {t('droid.settings.sessionSettings.showThinkingInMainView')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t(
                    'droid.settings.sessionSettings.showThinkingInMainViewDescription'
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

          {/* Compaction Settings Section */}
          <div className="space-y-4 pt-4 border-t">
            <div className="flex flex-col gap-1">
              <h2 className="text-base font-medium">
                {t('droid.settings.compaction.title')}
              </h2>
              <p className="text-sm text-muted-foreground">
                {t('droid.settings.compaction.description')}
              </p>
            </div>

            {/* Compaction Model Mode */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label
                    htmlFor="compaction-model-mode"
                    className="text-sm font-medium"
                  >
                    {t('droid.settings.compaction.modelMode')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t('droid.settings.compaction.modelModeDescription')}
                  </p>
                </div>
                <Select
                  value={compactionModelMode}
                  onValueChange={handleCompactionModelModeChange}
                >
                  <SelectTrigger className="w-48">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="current-model">
                      {t('droid.settings.compaction.modelMode.currentModel')}
                    </SelectItem>
                    <SelectItem value="factory-default">
                      {t('droid.settings.compaction.modelMode.factoryDefault')}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Compaction Token Limit */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label
                    htmlFor="compaction-token-limit"
                    className="text-sm font-medium"
                  >
                    {t('droid.settings.compaction.tokenLimit')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t('droid.settings.compaction.tokenLimitDescription')}
                  </p>
                </div>
                <TokenLimitSelect
                  value={compactionTokenLimit}
                  onValueChange={handleCompactionTokenLimitChange}
                />
              </div>
            </div>

            {/* Per-Model Token Limit Overrides */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex flex-col gap-1">
                  <Label className="text-sm font-medium">
                    {t('droid.settings.compaction.perModel.title')}
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    {t('droid.settings.compaction.perModel.description')}
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleAddPerModelOverride}
                >
                  <Plus className="h-4 w-4 mr-1" />
                  {t('droid.settings.compaction.perModel.addOverride')}
                </Button>
              </div>

              {perModelOverrides.length > 0 && (
                <div className="space-y-2">
                  {perModelOverrides.map((override, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2 p-2 bg-muted/50 rounded-md"
                    >
                      <Input
                        className="flex-1 text-sm"
                        placeholder={t(
                          'droid.settings.compaction.perModel.modelIdPlaceholder'
                        )}
                        value={override.modelId}
                        onChange={e =>
                          handlePerModelOverrideChange(
                            index,
                            'modelId',
                            e.target.value
                          )
                        }
                        onBlur={() => savePerModelOverrides(perModelOverrides)}
                      />
                      <TokenLimitSelect
                        value={override.tokenLimit}
                        onValueChange={v =>
                          handlePerModelOverrideChange(index, 'tokenLimit', v)
                        }
                        className="w-48"
                      />
                      <Button
                        variant="ghost"
                        size="icon"
                        className="shrink-0 text-destructive hover:text-destructive"
                        onClick={() => handleRemovePerModelOverride(index)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Disable Auto Update Dialog */}
      <Dialog
        open={disableAutoUpdateDialogOpen}
        onOpenChange={setDisableAutoUpdateDialogOpen}
      >
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>
              {t('droid.settings.disableAutoUpdate.title')}
            </DialogTitle>
            <DialogDescription>
              {t('droid.settings.disableAutoUpdate.description')}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-md">
              <p className="text-sm text-yellow-600 dark:text-yellow-500">
                ⚠️ {t('droid.settings.disableAutoUpdate.warning')}
              </p>
            </div>

            <div className="space-y-4 pt-2">
              <p className="text-sm font-medium">
                {t('droid.settings.disableAutoUpdate.instructions.title')}
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
