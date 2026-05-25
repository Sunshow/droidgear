import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  AlertCircle,
  CloudDownload,
  Feather,
  Loader2,
  Plus,
  RefreshCw,
  Trash2,
  Undo2,
} from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { SecretInput } from '@/components/ui/secret-input'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Badge } from '@/components/ui/badge'
import type { JsonValue } from '@/lib/bindings'
import {
  CLAUDE_AUTH_TOKEN_ENV,
  CLAUDE_BASE_URL_ENV,
  CLAUDE_MODEL_ENV,
  CLAUDE_SMALL_MODEL_ENV,
  type ClaudeReasoningEffort,
  type ClaudeThinkingMode,
  getEnvString,
  getReasoningEffort,
  getThinkingMode,
  isSmallModelMirroringMain,
  setEnvString,
  setReasoningEffort,
  setSmallModelMirroring,
  setThinkingMode,
} from '@/lib/claude-settings-mapping'
import { hasOpaqueClaudeModelId } from '@/lib/utils'
import {
  type ClaudeSettingsDoc,
  useClaudeSettingsStore,
} from '@/store/claude-settings-store'
import { ImportFromChannelDialog } from './ImportFromChannelDialog'

const PRESET_ENV_VARS: { key: string; defaultValue: string }[] = [
  { key: 'CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC', defaultValue: '1' },
  { key: 'DISABLE_AUTOUPDATER', defaultValue: '0' },
  { key: 'DISABLE_BUG_COMMAND', defaultValue: '0' },
  { key: 'DISABLE_COST_WARNINGS', defaultValue: '0' },
  { key: 'DISABLE_ERROR_REPORTING', defaultValue: '0' },
  { key: 'DISABLE_NON_ESSENTIAL_MODEL_CALLS', defaultValue: '0' },
  { key: 'DISABLE_TELEMETRY', defaultValue: '0' },
]

const DEFAULT_MODE_VALUES = [
  'default',
  'acceptEdits',
  'plan',
  'auto',
  'dontAsk',
  'bypassPermissions',
] as const

const DEFAULT_MODE_UNSET = '__unset__'
const DISABLE_BYPASS_UNSET = '__unset__'

interface EnvEntry {
  key: string
  value: string
}

function getEnvEntries(doc: ClaudeSettingsDoc | null): EnvEntry[] {
  if (!doc) return []
  const env = doc.env
  if (!env || typeof env !== 'object' || Array.isArray(env)) return []
  return Object.entries(env as Record<string, JsonValue>).map(([k, v]) => ({
    key: k,
    value:
      typeof v === 'string'
        ? v
        : v == null
          ? ''
          : typeof v === 'number' || typeof v === 'boolean'
            ? String(v)
            : JSON.stringify(v),
  }))
}

function envEntriesToObject(entries: EnvEntry[]): Record<string, JsonValue> {
  const env: Record<string, JsonValue> = {}
  for (const { key, value } of entries) {
    if (!key.trim()) continue
    env[key] = value
  }
  return env
}

function asString(value: JsonValue | undefined): string | null {
  return typeof value === 'string' ? value : null
}

function asBool(value: JsonValue | undefined): boolean | null {
  return typeof value === 'boolean' ? value : null
}

function asNumber(value: JsonValue | undefined): number | null {
  return typeof value === 'number' ? value : null
}

function getPermissionsObject(
  doc: ClaudeSettingsDoc
): Record<string, JsonValue> {
  const value = doc.permissions
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return { ...(value as Record<string, JsonValue>) }
  }
  return {}
}

function omitKey(
  source: Record<string, JsonValue>,
  key: string
): Record<string, JsonValue> {
  const next: Record<string, JsonValue> = {}
  for (const k of Object.keys(source)) {
    if (k !== key) next[k] = source[k] as JsonValue
  }
  return next
}

export function ClaudeSettingsPage() {
  const { t } = useTranslation()
  const activeFile = useClaudeSettingsStore(state => state.activeFile)
  const currentJson = useClaudeSettingsStore(state => state.currentJson)
  const hasChanges = useClaudeSettingsStore(state => state.hasChanges)
  const isLoading = useClaudeSettingsStore(state => state.isLoading)
  const error = useClaudeSettingsStore(state => state.error)
  const loadFiles = useClaudeSettingsStore(state => state.loadFiles)
  const patchJson = useClaudeSettingsStore(state => state.patchJson)
  const saveFile = useClaudeSettingsStore(state => state.saveFile)
  const resetChanges = useClaudeSettingsStore(state => state.resetChanges)
  const setError = useClaudeSettingsStore(state => state.setError)

  if (!activeFile || !currentJson) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        {isLoading ? (
          <div className="flex items-center gap-2 text-sm">
            <Loader2 className="h-4 w-4 animate-spin" />
            {t('common.loading')}
          </div>
        ) : (
          <p className="text-sm">{t('claude.settings.empty')}</p>
        )}
      </div>
    )
  }

  const handleSave = async () => {
    await saveFile()
    const latestError = useClaudeSettingsStore.getState().error
    if (latestError) {
      toast.error(latestError)
      return
    }
    toast.success(t('claude.settings.saveSuccess'))
  }

  const handleReset = async () => {
    await resetChanges()
    toast.info(t('claude.settings.resetDone'))
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">
            {t('claude.settings.title')}
          </h1>
          <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
            <Badge variant="outline">{activeFile.name}</Badge>
            <span className="truncate font-mono">{activeFile.path}</span>
          </div>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Button
            variant="outline"
            size="icon"
            onClick={() => loadFiles()}
            disabled={isLoading}
            title={t('common.refresh')}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            onClick={handleReset}
            disabled={!hasChanges}
          >
            <Undo2 className="h-4 w-4 mr-2" />
            {t('claude.settings.discard')}
          </Button>
          <Button onClick={handleSave} disabled={!hasChanges}>
            {t('common.save')}
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

      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        <ProviderSection json={currentJson} patchJson={patchJson} />
        <ModelSection json={currentJson} patchJson={patchJson} />
        <ReasoningThinkingSection json={currentJson} patchJson={patchJson} />
        <GeneralSection json={currentJson} patchJson={patchJson} />
        <PermissionsSection
          json={currentJson}
          activeFileIsGlobal={activeFile.isGlobal}
          patchJson={patchJson}
        />
        <EnvSection
          key={activeFile.name}
          json={currentJson}
          patchJson={patchJson}
        />
      </div>
    </div>
  )
}

interface SectionProps {
  json: ClaudeSettingsDoc
  patchJson: (mutator: (draft: ClaudeSettingsDoc) => void) => void
}

function GeneralSection({ json, patchJson }: SectionProps) {
  const { t } = useTranslation()
  const includeCoAuthoredBy = asBool(json.includeCoAuthoredBy) ?? true
  const autoUpdate = asBool(json.autoUpdate) ?? true
  const cleanupPeriodDays = asNumber(json.cleanupPeriodDays) ?? 30

  return (
    <section className="space-y-4">
      <div>
        <h2 className="text-base font-medium">
          {t('claude.settings.general.title')}
        </h2>
        <p className="text-xs text-muted-foreground mt-1">
          {t('claude.settings.general.hint')}
        </p>
      </div>

      <div className="flex items-center justify-between">
        <div className="flex flex-col gap-1">
          <Label className="text-sm font-medium">
            {t('claude.settings.general.autoUpdate')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('claude.settings.general.autoUpdateHint')}
          </p>
        </div>
        <Switch
          checked={autoUpdate}
          onCheckedChange={value =>
            patchJson(draft => {
              if (value) {
                draft.autoUpdate = undefined as unknown as JsonValue
                Reflect.deleteProperty(draft, 'autoUpdate')
              } else {
                draft.autoUpdate = false
              }
            })
          }
        />
      </div>

      <div className="flex items-center justify-between">
        <div className="flex flex-col gap-1">
          <Label className="text-sm font-medium">
            {t('claude.settings.general.includeCoAuthored')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('claude.settings.general.includeCoAuthoredHint')}
          </p>
        </div>
        <Switch
          checked={includeCoAuthoredBy}
          onCheckedChange={value =>
            patchJson(draft => {
              if (value) {
                Reflect.deleteProperty(draft, 'includeCoAuthoredBy')
              } else {
                draft.includeCoAuthoredBy = false
              }
            })
          }
        />
      </div>

      <div className="flex items-center justify-between gap-4">
        <div className="flex flex-col gap-1">
          <Label className="text-sm font-medium">
            {t('claude.settings.general.cleanupPeriodDays')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('claude.settings.general.cleanupPeriodDaysHint')}
          </p>
        </div>
        <Input
          type="number"
          min={1}
          max={3650}
          className="w-28"
          value={cleanupPeriodDays}
          onChange={e => {
            const next = Number(e.target.value)
            patchJson(draft => {
              if (!Number.isFinite(next) || next === 30) {
                Reflect.deleteProperty(draft, 'cleanupPeriodDays')
              } else {
                draft.cleanupPeriodDays = next
              }
            })
          }}
        />
      </div>
    </section>
  )
}

interface PermissionsSectionProps extends SectionProps {
  activeFileIsGlobal: boolean
}

function PermissionsSection({
  json,
  activeFileIsGlobal,
  patchJson,
}: PermissionsSectionProps) {
  const { t } = useTranslation()
  const permissions = useMemo(() => getPermissionsObject(json), [json])
  const defaultMode = asString(permissions.defaultMode)
  const disableBypass = asString(json.disableBypassPermissionsMode)
  const skipDangerousPrompt =
    asBool(json.skipDangerousModePermissionPrompt) ?? false

  return (
    <section className="space-y-4 pt-4 border-t">
      <div>
        <h2 className="text-base font-medium">
          {t('claude.settings.permissions.title')}
        </h2>
        <p className="text-xs text-muted-foreground mt-1">
          {t('claude.settings.permissions.hint')}
        </p>
      </div>

      <div className="flex items-center justify-between gap-4">
        <div className="flex flex-col gap-1">
          <Label className="text-sm font-medium">
            {t('claude.settings.permissions.defaultMode')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('claude.settings.permissions.defaultModeHint')}
          </p>
        </div>
        <Select
          value={defaultMode ?? DEFAULT_MODE_UNSET}
          onValueChange={value =>
            patchJson(draft => {
              const nextPermissions = getPermissionsObject(draft)
              const updated =
                value === DEFAULT_MODE_UNSET
                  ? omitKey(nextPermissions, 'defaultMode')
                  : { ...nextPermissions, defaultMode: value }
              if (Object.keys(updated).length === 0) {
                Reflect.deleteProperty(draft, 'permissions')
              } else {
                draft.permissions = updated
              }
            })
          }
        >
          <SelectTrigger className="w-44">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={DEFAULT_MODE_UNSET}>
              {t('claude.settings.permissions.unset')}
            </SelectItem>
            {DEFAULT_MODE_VALUES.map(value => (
              <SelectItem key={value} value={value}>
                {value}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="flex items-center justify-between gap-4">
        <div className="flex flex-col gap-1">
          <Label className="text-sm font-medium">
            {t('claude.settings.permissions.disableBypass')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('claude.settings.permissions.disableBypassHint')}
          </p>
        </div>
        <Select
          value={disableBypass ?? DISABLE_BYPASS_UNSET}
          onValueChange={value =>
            patchJson(draft => {
              if (value === DISABLE_BYPASS_UNSET) {
                Reflect.deleteProperty(draft, 'disableBypassPermissionsMode')
              } else {
                draft.disableBypassPermissionsMode = value
              }
            })
          }
        >
          <SelectTrigger className="w-44">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={DISABLE_BYPASS_UNSET}>
              {t('claude.settings.permissions.unset')}
            </SelectItem>
            <SelectItem value="disable">disable</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="flex items-center justify-between gap-4">
        <div className="flex flex-col gap-1">
          <Label className="text-sm font-medium">
            {t('claude.settings.permissions.skipDangerousPrompt')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('claude.settings.permissions.skipDangerousPromptHint')}
          </p>
          {!activeFileIsGlobal && (
            <div className="flex items-start gap-2 text-amber-600 dark:text-amber-500 text-xs mt-1">
              <AlertCircle className="h-3 w-3 shrink-0 mt-0.5" />
              <span>
                {t('claude.settings.permissions.skipDangerousProjectWarning')}
              </span>
            </div>
          )}
        </div>
        <Switch
          checked={skipDangerousPrompt}
          onCheckedChange={value =>
            patchJson(draft => {
              if (value) {
                draft.skipDangerousModePermissionPrompt = true
              } else {
                Reflect.deleteProperty(
                  draft,
                  'skipDangerousModePermissionPrompt'
                )
              }
            })
          }
        />
      </div>
    </section>
  )
}

function EnvSection({ json, patchJson }: SectionProps) {
  const { t } = useTranslation()
  const [entries, setEntries] = useState<EnvEntry[]>(() => getEnvEntries(json))

  const commit = (next: EnvEntry[]) => {
    setEntries(next)
    patchJson(draft => {
      const updated = envEntriesToObject(next)
      if (Object.keys(updated).length === 0) {
        Reflect.deleteProperty(draft, 'env')
      } else {
        draft.env = updated
      }
    })
  }

  const handleAdd = () => {
    commit([...entries, { key: '', value: '' }])
  }

  const handleInsertPreset = (key: string, defaultValue: string) => {
    if (entries.some(entry => entry.key === key)) {
      toast.warning(t('claude.settings.envEditor.alreadyPresent', { key }))
      return
    }
    commit([...entries, { key, value: defaultValue }])
  }

  return (
    <section className="space-y-4 pt-4 border-t">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-base font-medium">
            {t('claude.settings.envEditor.title')}
          </h2>
          <p className="text-xs text-muted-foreground mt-1">
            {t('claude.settings.envEditor.hint')}
          </p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm">
                {t('claude.settings.envEditor.insertPreset')}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="min-w-[260px]">
              {PRESET_ENV_VARS.map(preset => (
                <DropdownMenuItem
                  key={preset.key}
                  onClick={() =>
                    handleInsertPreset(preset.key, preset.defaultValue)
                  }
                >
                  <span className="font-mono text-xs">{preset.key}</span>
                  <span className="ml-auto text-[10px] text-muted-foreground">
                    {`= "${preset.defaultValue}"`}
                  </span>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
          <Button variant="outline" size="sm" onClick={handleAdd}>
            <Plus className="h-4 w-4 mr-1" />
            {t('claude.settings.envEditor.add')}
          </Button>
        </div>
      </div>

      {entries.length === 0 && (
        <p className="text-xs text-muted-foreground italic">
          {t('claude.settings.envEditor.empty')}
        </p>
      )}

      <div className="space-y-2">
        {entries.map((entry, index) => (
          <div
            key={index}
            className="flex items-center gap-2 p-2 bg-muted/40 rounded-md"
          >
            <Input
              className="flex-1 font-mono text-xs"
              placeholder={t('claude.settings.envEditor.keyPlaceholder')}
              value={entry.key}
              onChange={e => {
                const next = entries.map((row, i) =>
                  i === index ? { ...row, key: e.target.value } : row
                )
                commit(next)
              }}
            />
            <Input
              className="flex-1 font-mono text-xs"
              placeholder={t('claude.settings.envEditor.valuePlaceholder')}
              value={entry.value}
              onChange={e => {
                const next = entries.map((row, i) =>
                  i === index ? { ...row, value: e.target.value } : row
                )
                commit(next)
              }}
            />
            <Button
              variant="ghost"
              size="icon"
              className="shrink-0 text-destructive hover:text-destructive"
              onClick={() => {
                commit(entries.filter((_, i) => i !== index))
              }}
              title={t('common.delete')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        ))}
      </div>
    </section>
  )
}

function ProviderSection({ json, patchJson }: SectionProps) {
  const { t } = useTranslation()
  const baseUrl = getEnvString(json, CLAUDE_BASE_URL_ENV) ?? ''
  const bearerToken = getEnvString(json, CLAUDE_AUTH_TOKEN_ENV) ?? ''
  const [importDialogOpen, setImportDialogOpen] = useState(false)

  const handleImported = (result: {
    baseUrl: string
    apiKey: string
    defaultModel?: string
  }) => {
    patchJson(draft => {
      setEnvString(draft, CLAUDE_BASE_URL_ENV, result.baseUrl)
      setEnvString(draft, CLAUDE_AUTH_TOKEN_ENV, result.apiKey)
      if (result.defaultModel) {
        const wasMirroring = isSmallModelMirroringMain(draft)
        setEnvString(draft, CLAUDE_MODEL_ENV, result.defaultModel)
        if (wasMirroring) {
          setEnvString(draft, CLAUDE_SMALL_MODEL_ENV, result.defaultModel)
        }
      }
    })
    toast.success(t('claude.provider.importDialog.imported'))
  }

  return (
    <section className="space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-base font-medium">
            {t('claude.provider.title')}
          </h2>
          <p className="text-xs text-muted-foreground mt-1">
            {t('claude.provider.hint')}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          className="shrink-0"
          onClick={() => setImportDialogOpen(true)}
          title={t('claude.provider.importFromChannel')}
        >
          <CloudDownload className="h-4 w-4 mr-2" />
          {t('claude.provider.importFromChannel')}
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="claude-provider-base-url">
            {t('claude.provider.baseUrl')}
          </Label>
          <Input
            id="claude-provider-base-url"
            value={baseUrl}
            onChange={e =>
              patchJson(draft =>
                setEnvString(draft, CLAUDE_BASE_URL_ENV, e.target.value || null)
              )
            }
            placeholder={t('claude.provider.baseUrlPlaceholder')}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="claude-provider-bearer-token">
            {t('claude.provider.bearerToken')}
          </Label>
          <SecretInput
            id="claude-provider-bearer-token"
            value={bearerToken}
            onChange={e =>
              patchJson(draft =>
                setEnvString(
                  draft,
                  CLAUDE_AUTH_TOKEN_ENV,
                  e.target.value || null
                )
              )
            }
            placeholder={t('claude.provider.bearerTokenPlaceholder')}
          />
        </div>
      </div>

      <ImportFromChannelDialog
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        onImported={handleImported}
      />
    </section>
  )
}

function ModelSection({ json, patchJson }: SectionProps) {
  const { t } = useTranslation()
  const model = getEnvString(json, CLAUDE_MODEL_ENV) ?? ''
  const smallModel = getEnvString(json, CLAUDE_SMALL_MODEL_ENV) ?? ''
  const mirror = isSmallModelMirroringMain(json)
  const showOpaqueWarning = hasOpaqueClaudeModelId(model)
  const smallModelDisplay = mirror ? model : smallModel

  return (
    <section className="space-y-4 pt-4 border-t">
      <div>
        <h2 className="text-base font-medium">{t('claude.model.title')}</h2>
        <p className="text-xs text-muted-foreground mt-1">
          {t('claude.model.hint')}
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="claude-model-name">{t('claude.model.name')}</Label>
          <Input
            id="claude-model-name"
            value={model}
            onChange={e => {
              const next = e.target.value || null
              patchJson(draft => {
                const wasMirroring = isSmallModelMirroringMain(draft)
                setEnvString(draft, CLAUDE_MODEL_ENV, next)
                if (wasMirroring) {
                  setEnvString(draft, CLAUDE_SMALL_MODEL_ENV, next)
                }
              })
            }}
            placeholder={t('claude.model.namePlaceholder')}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="claude-small-model-name">
            {t('claude.model.smallModel')}
          </Label>
          <Input
            id="claude-small-model-name"
            value={smallModelDisplay}
            disabled={mirror}
            onChange={e =>
              patchJson(draft =>
                setEnvString(
                  draft,
                  CLAUDE_SMALL_MODEL_ENV,
                  e.target.value || null
                )
              )
            }
            placeholder={t('claude.model.smallModelPlaceholder')}
          />
        </div>
      </div>

      {showOpaqueWarning && (
        <div className="flex items-start gap-3 rounded-md border border-amber-500/30 bg-amber-500/10 p-3">
          <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-amber-600" />
          <div className="space-y-1">
            <p className="text-sm font-medium text-amber-700">
              {t('claude.model.capabilityWarningTitle')}
            </p>
            <p className="text-xs text-amber-700/90">
              {t('claude.model.capabilityWarningBody', { model })}
            </p>
          </div>
        </div>
      )}

      <div className="flex items-start gap-3 rounded-md border p-3">
        <Checkbox
          id="claude-small-model-mirror"
          checked={mirror}
          onCheckedChange={checked => {
            const next = checked === true
            patchJson(draft => {
              const mainModel = getEnvString(draft, CLAUDE_MODEL_ENV)
              if (next) {
                setSmallModelMirroring(draft, true, mainModel)
              } else if (mainModel) {
                setEnvString(draft, CLAUDE_SMALL_MODEL_ENV, mainModel)
              }
            })
          }}
        />
        <div className="space-y-1">
          <Label
            htmlFor="claude-small-model-mirror"
            className="text-sm font-medium"
          >
            {t('claude.model.smallModelUsesMainModel')}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t('claude.model.smallModelUsesMainModelHint')}
          </p>
        </div>
      </div>
    </section>
  )
}

const REASONING_VALUES: readonly ClaudeReasoningEffort[] = [
  'inherit',
  'low',
  'medium',
  'high',
  'max',
]

const THINKING_VALUES: readonly ClaudeThinkingMode[] = ['inherit', 'on', 'off']

function ReasoningThinkingSection({ json, patchJson }: SectionProps) {
  const { t } = useTranslation()
  const effort = getReasoningEffort(json)
  const thinking = getThinkingMode(json)

  return (
    <section className="space-y-4 pt-4 border-t">
      <div className="flex items-center gap-2">
        <Feather className="h-4 w-4 text-muted-foreground" />
        <h2 className="text-base font-medium">{t('claude.reasoning.title')}</h2>
      </div>
      <p className="text-xs text-muted-foreground">
        {t('claude.reasoning.hint')}
      </p>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="space-y-2">
          <Label>{t('claude.reasoning.effort')}</Label>
          <Select
            value={effort}
            onValueChange={value =>
              patchJson(draft =>
                setReasoningEffort(draft, value as ClaudeReasoningEffort)
              )
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {REASONING_VALUES.map(value => (
                <SelectItem key={value} value={value}>
                  {t(`claude.reasoning.${value}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <Label>{t('claude.thinking.mode')}</Label>
          <Select
            value={thinking}
            onValueChange={value =>
              patchJson(draft =>
                setThinkingMode(draft, value as ClaudeThinkingMode)
              )
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {THINKING_VALUES.map(value => (
                <SelectItem key={value} value={value}>
                  {t(`claude.thinking.${value}`)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </section>
  )
}
