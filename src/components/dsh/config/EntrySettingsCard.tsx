import { useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ChevronDown,
  ChevronRight,
  LoaderCircle,
  RotateCcw,
  Save,
  Undo2,
} from 'lucide-react'
import { toast } from 'sonner'
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
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'
import type {
  DshConfigEntryView,
  DshConfigFieldView,
  JsonValue,
} from '@/lib/bindings'
import {
  type JsonObject,
  entryTitleKey,
  findField,
  isJsExpression,
  isJsonObject,
  isSecretMarker,
  jsonEquals,
  jsonText,
  layerLabelKey,
  tierLabelKey,
} from '../dsh-config-utils'
import { EnumField, type EnumOption } from './EnumField'
import { JsExpressionBadge } from './JsExpressionBadge'
import { NumberField } from './NumberField'
import { OverrideBadge } from './OverrideBadge'
import { RestartBadge } from './RestartBadge'
import { SecretBadge } from './SecretBadge'
import { SettingsRow } from './SettingsRow'
import { SwitchField } from './SwitchField'
import { TextField } from './TextField'

/** Per-field presentation overrides for curated surfaces. */
export interface FieldOverride {
  /** Render the field as another control kind (e.g. a string as a select). */
  kind?: 'string' | 'number' | 'boolean' | 'enum'
  /** Fixed options for an overridden enum control. */
  options?: EnumOption[]
}

interface EntrySettingsCardProps {
  entry: DshConfigEntryView
  /** Card title; defaults to the curated i18n title or the entry description. */
  title?: ReactNode
  /** Render only these fields, in this order. Defaults to every field. */
  fields?: readonly string[]
  /** Translated labels keyed by field name. */
  labels?: Record<string, ReactNode>
  /** Translated hints keyed by field name (overrides the schema description). */
  hints?: Record<string, ReactNode>
  fieldOverrides?: Record<string, FieldOverride>
  /** Field tiers rendered as editable controls (C tier is always read-only). */
  editableTiers?: readonly string[]
  /** Renders a toggle that enables/disables the entry. */
  showDisabledToggle?: boolean
  /** Renders a collapsible header (closed by default). */
  collapsible?: boolean
  defaultOpen?: boolean
  onSave: (config: JsonValue | null) => Promise<void>
  onToggleDisabled?: (disabled: boolean) => Promise<void>
  saving?: boolean
  /** Translator for enum values; defaults to the raw value. */
  optionLabel?: (value: string) => string
}

const DEFAULT_EDITABLE_TIERS: readonly string[] = ['A', 'B']

const EMPTY_DRAFTS: Record<string, JsonValue> = {}

function toastWriteError(error: unknown) {
  toast.error(error instanceof Error ? error.message : String(error))
}

function readOnlyText(field: DshConfigFieldView, value: JsonValue): string {
  if (field.secret || isSecretMarker(value)) return '••••••••'
  if (isJsExpression(value) && isJsonObject(value))
    return `!!js ${String(value['$js'])}`
  return jsonText(value)
}

/** One composed entry rendered as a settings form. */
export function EntrySettingsCard({
  entry,
  title,
  fields,
  labels,
  hints,
  fieldOverrides,
  editableTiers = DEFAULT_EDITABLE_TIERS,
  showDisabledToggle,
  collapsible,
  defaultOpen,
  onSave,
  onToggleDisabled,
  saving,
  optionLabel,
}: EntrySettingsCardProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(defaultOpen ?? !collapsible)
  const [restoreOpen, setRestoreOpen] = useState(false)

  // Local edits are keyed to the snapshot's effective config: a reload that
  // replaces the entry with new data starts a fresh, empty draft.
  const configKey = JSON.stringify(entry.config ?? null)
  const [draftState, setDraftState] = useState<{
    key: string
    values: Record<string, JsonValue>
  }>({ key: configKey, values: EMPTY_DRAFTS })
  const drafts = draftState.key === configKey ? draftState.values : EMPTY_DRAFTS

  const titleKey = entryTitleKey(entry.id)
  const heading =
    title ?? (titleKey ? t(titleKey) : entry.description || entry.id)
  const visibleFields = fields
    ? fields
        .map(name => findField(entry, name))
        .filter((field): field is DshConfigFieldView => field !== undefined)
    : entry.fields
  const dirty = Object.keys(drafts).length > 0
  const disabled = saving === true
  const shadowed = entry.shadowedByHome

  const hasDraft = (name: string) =>
    Object.prototype.hasOwnProperty.call(drafts, name)

  const clearDrafts = () => {
    setDraftState({ key: configKey, values: EMPTY_DRAFTS })
  }

  const setDraft = (field: DshConfigFieldView, value: JsonValue) => {
    setDraftState(prev => {
      const values = prev.key === configKey ? prev.values : EMPTY_DRAFTS
      const next: Record<string, JsonValue> = {}
      for (const [name, existing] of Object.entries(values)) {
        if (name !== field.name) next[name] = existing
      }
      if (!jsonEquals(value, field.value)) next[field.name] = value
      return { key: configKey, values: next }
    })
  }

  const buildConfig = (): JsonValue => {
    const base = isJsonObject(entry.config) ? entry.config : {}
    const removed = new Set<string>()
    for (const [name, value] of Object.entries(drafts)) {
      if (value === null) removed.add(name)
    }
    const config: JsonObject = {}
    for (const [name, value] of Object.entries(base)) {
      if (!removed.has(name)) config[name] = value
    }
    for (const [name, value] of Object.entries(drafts)) {
      if (value !== null) config[name] = value
    }
    return config
  }

  const handleSave = async () => {
    try {
      await onSave(buildConfig())
      clearDrafts()
      toast.success(t('dsh.config.saved'))
    } catch (error) {
      toastWriteError(error)
    }
  }

  const handleRestore = async () => {
    try {
      await onSave(null)
      clearDrafts()
      toast.success(t('dsh.config.saved'))
    } catch (error) {
      toastWriteError(error)
    }
  }

  const handleToggleDisabled = async (next: boolean) => {
    if (!onToggleDisabled) return
    try {
      await onToggleDisabled(next)
      toast.success(t('dsh.config.saved'))
    } catch (error) {
      toastWriteError(error)
    }
  }

  const badgeList = (
    <>
      <Badge variant="outline" className="text-xs text-muted-foreground">
        {t(layerLabelKey(entry.layer))}
      </Badge>
      <Badge
        variant="outline"
        className="text-xs text-muted-foreground"
        title={entry.tier}
      >
        {t(tierLabelKey(entry.tier))}
      </Badge>
      {entry.hasJs ? <JsExpressionBadge /> : null}
      {entry.overridden ? <OverrideBadge /> : null}
      {entry.disabled && !showDisabledToggle ? (
        <Badge variant="secondary" className="text-xs">
          {t('dsh.config.disabled')}
        </Badge>
      ) : null}
    </>
  )

  const headingNode = collapsible ? (
    <button
      type="button"
      className="flex min-w-0 items-center gap-1.5 text-sm font-medium hover:underline"
      aria-expanded={open}
      onClick={() => setOpen(value => !value)}
    >
      {open ? (
        <ChevronDown className="size-4 shrink-0" aria-hidden="true" />
      ) : (
        <ChevronRight className="size-4 shrink-0" aria-hidden="true" />
      )}
      <span className="truncate">{heading}</span>
    </button>
  ) : (
    <h3 className="text-sm font-medium">{heading}</h3>
  )

  return (
    <section
      data-entry-id={entry.id}
      className="rounded-lg border bg-card text-card-foreground"
    >
      <header className="flex items-start justify-between gap-3 p-4">
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            {headingNode}
            <code className="rounded bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
              {entry.id}
            </code>
            {badgeList}
          </div>
          {entry.description ? (
            <p className="text-xs text-muted-foreground">{entry.description}</p>
          ) : null}
          {shadowed ? (
            <p className="text-xs text-destructive">
              {t('dsh.config.homePatchShadow')}
            </p>
          ) : null}
        </div>
        <div className="flex shrink-0 items-center gap-3">
          {saving ? (
            <LoaderCircle
              className="size-4 animate-spin text-muted-foreground"
              aria-label={t('dsh.config.saving')}
            />
          ) : null}
          {showDisabledToggle && onToggleDisabled ? (
            <label
              className={cn(
                'flex items-center gap-2 text-xs text-muted-foreground',
                disabled && 'opacity-50'
              )}
              htmlFor={`dsh-disabled-${entry.id}`}
            >
              <Switch
                id={`dsh-disabled-${entry.id}`}
                checked={entry.disabled}
                disabled={disabled}
                onCheckedChange={next => {
                  void handleToggleDisabled(next)
                }}
              />
              {t('dsh.config.disabled')}
            </label>
          ) : null}
        </div>
      </header>

      {open ? (
        <div className="space-y-4 border-t p-4">
          {visibleFields.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t('dsh.config.noEntries')}
            </p>
          ) : (
            visibleFields.map(field => {
              const override = fieldOverrides?.[field.name]
              const kind = override?.kind ?? field.kind
              const value = hasDraft(field.name)
                ? (drafts[field.name] ?? null)
                : field.value
              const js = isJsExpression(field.value)
              const tierEditable = editableTiers.includes(field.tier)
              const readOnly =
                field.secret || js || kind === 'json' || !tierEditable
              const fieldId = `dsh-field-${entry.id}-${field.name}`
              const fieldDirty = hasDraft(field.name)

              let control: ReactNode
              if (readOnly) {
                const text = readOnlyText(field, value)
                control = (
                  <span
                    className="block max-w-md truncate rounded-md bg-muted px-2 py-1 font-mono text-xs"
                    title={text}
                  >
                    {text || t('dsh.field.enumEmpty')}
                  </span>
                )
              } else if (kind === 'boolean') {
                control = (
                  <SwitchField
                    id={fieldId}
                    value={value === true}
                    disabled={disabled}
                    dirty={fieldDirty}
                    onChange={next => setDraft(field, next)}
                  />
                )
              } else if (kind === 'number') {
                control = (
                  <NumberField
                    id={fieldId}
                    value={typeof value === 'number' ? value : null}
                    disabled={disabled}
                    dirty={fieldDirty}
                    onChange={next => setDraft(field, next)}
                  />
                )
              } else if (kind === 'enum') {
                const options = buildEnumOptions(
                  field,
                  value,
                  override?.options,
                  optionLabel
                )
                control = (
                  <EnumField
                    id={fieldId}
                    value={typeof value === 'string' ? value : null}
                    options={options}
                    disabled={disabled}
                    dirty={fieldDirty}
                    onChange={next => setDraft(field, next)}
                  />
                )
              } else if (kind === 'string') {
                control = (
                  <TextField
                    id={fieldId}
                    value={typeof value === 'string' ? value : ''}
                    disabled={disabled}
                    dirty={fieldDirty}
                    onChange={next => setDraft(field, next)}
                  />
                )
              } else {
                const text = readOnlyText(field, value)
                control = (
                  <span
                    className="block max-w-md truncate rounded-md bg-muted px-2 py-1 font-mono text-xs"
                    title={text}
                  >
                    {text || t('dsh.field.enumEmpty')}
                  </span>
                )
              }

              return (
                <SettingsRow
                  key={field.name}
                  htmlFor={fieldId}
                  label={labels?.[field.name] ?? field.name}
                  description={
                    hints?.[field.name] ?? (field.description || undefined)
                  }
                  badges={
                    <>
                      {!field.volatile ? <RestartBadge /> : null}
                      {field.secret ? <SecretBadge /> : null}
                      {js ? <JsExpressionBadge /> : null}
                      {field.overridden ? <OverrideBadge /> : null}
                      {readOnly && !field.secret && !js ? (
                        <Badge
                          variant="outline"
                          className="text-xs text-muted-foreground"
                        >
                          {t('dsh.config.readonly')}
                        </Badge>
                      ) : null}
                    </>
                  }
                >
                  {control}
                </SettingsRow>
              )
            })
          )}

          <div className="flex flex-wrap items-center justify-end gap-2 border-t pt-3">
            {dirty ? (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={clearDrafts}
                disabled={disabled}
              >
                <Undo2 aria-hidden="true" />
                {t('dsh.config.discard')}
              </Button>
            ) : null}
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={!entry.overridden || disabled}
              onClick={() => setRestoreOpen(true)}
            >
              <RotateCcw aria-hidden="true" />
              {t('dsh.config.reset')}
            </Button>
            <Button
              type="button"
              size="sm"
              disabled={!dirty || disabled || shadowed}
              onClick={() => {
                void handleSave()
              }}
            >
              {saving ? (
                <LoaderCircle
                  className="animate-spin"
                  aria-hidden="true"
                  data-icon="inline-start"
                />
              ) : (
                <Save aria-hidden="true" />
              )}
              {saving ? t('dsh.config.saving') : t('dsh.config.save')}
            </Button>
          </div>
        </div>
      ) : null}

      <AlertDialog open={restoreOpen} onOpenChange={setRestoreOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('dsh.config.reset')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('dsh.config.resetConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                void handleRestore()
              }}
            >
              {t('dsh.config.reset')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  )
}

function buildEnumOptions(
  field: DshConfigFieldView,
  value: JsonValue,
  override: EnumOption[] | undefined,
  optionLabel: ((value: string) => string) | undefined
): EnumOption[] {
  const options =
    override ??
    field.enumValues.map(enumValue => ({
      value: enumValue,
      label: optionLabel ? optionLabel(enumValue) : enumValue,
    }))
  if (
    typeof value === 'string' &&
    value.length > 0 &&
    !options.some(option => option.value === value)
  ) {
    return [
      ...options,
      { value, label: optionLabel ? optionLabel(value) : value },
    ]
  }
  return options
}
