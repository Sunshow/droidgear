import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { KeyRound, LoaderCircle, Save, Trash2 } from 'lucide-react'
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
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { SecretInput } from '@/components/ui/secret-input'
import { useDshStore } from '@/store/dsh-store'
import type { DshConfigEntryView } from '@/lib/bindings'
import { findEntry, isJsonObject } from './dsh-config-utils'
import { isValidApiKeyEnv } from './provider-credentials'

const OFFICIAL_ENTRY_ID = 'llm-deepseek'
const DEFAULT_API_KEY_ENV = 'DEEPSEEK_API_KEY'

interface RefDraft {
  entryRef: string
  custom: string
  expanded: boolean
}

function readEntryApiKeyEnv(entry: DshConfigEntryView | undefined): string {
  if (!entry) return DEFAULT_API_KEY_ENV

  const config = isJsonObject(entry.config) ? entry.config : null
  const fromConfig = config?.['apiKeyEnv']
  if (typeof fromConfig === 'string' && fromConfig.trim()) {
    return fromConfig.trim()
  }

  const field = entry.fields.find(item => item.name === 'apiKeyEnv')
  if (typeof field?.value === 'string' && field.value.trim()) {
    return field.value.trim()
  }
  if (typeof field?.default === 'string' && field.default.trim()) {
    return field.default.trim()
  }
  return DEFAULT_API_KEY_ENV
}

function initialRefDraft(entryRef: string): RefDraft {
  const isCustom = entryRef !== DEFAULT_API_KEY_ENV
  return { entryRef, custom: isCustom ? entryRef : '', expanded: isCustom }
}

/**
 * `deepseek-official` API key card: writes `llm-deepseek.apiKeyEnv` and stores
 * the key in `~/.dsh/.credentials.yaml`.
 */
export function DshOfficialDeepseekCard() {
  const { t } = useTranslation()
  const snapshot = useDshStore(state => state.snapshot)
  const credentials = useDshStore(state => state.credentials)
  const saveEntry = useDshStore(state => state.saveEntry)
  const saveCredential = useDshStore(state => state.saveCredential)

  const entry = findEntry(snapshot, OFFICIAL_ENTRY_ID)
  const entryRef = readEntryApiKeyEnv(entry)

  const [refDraft, setRefDraft] = useState<RefDraft>(() =>
    initialRefDraft(entryRef)
  )
  const [secretValue, setSecretValue] = useState('')
  const [saving, setSaving] = useState(false)
  const [clearOpen, setClearOpen] = useState(false)

  // Re-initialize when the snapshot loads or the stored reference changes.
  const draft =
    refDraft.entryRef === entryRef ? refDraft : initialRefDraft(entryRef)
  const customRef = draft.custom
  const expanded = draft.expanded
  const effectiveRef = customRef.trim() || entryRef
  const refInvalid = customRef.trim().length > 0 && !isValidApiKeyEnv(customRef)
  const configured = Boolean(credentials[effectiveRef])
  const hasChanges = effectiveRef !== entryRef || secretValue.trim().length > 0

  const handleSave = async () => {
    if (refInvalid) return
    const key = secretValue.trim()
    setSaving(true)
    try {
      if (effectiveRef !== entryRef) {
        const base = isJsonObject(entry?.config) ? entry.config : {}
        await saveEntry(OFFICIAL_ENTRY_ID, {
          ...base,
          apiKeyEnv: effectiveRef,
        })
      }
      if (key) {
        await saveCredential(effectiveRef, key)
      }
      setSecretValue('')
      toast.success(t('dsh.officialDeepseek.saved'))
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error))
    } finally {
      setSaving(false)
    }
  }

  const handleClear = async () => {
    try {
      await saveCredential(effectiveRef, '')
      toast.success(t('dsh.officialDeepseek.cleared'))
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  }

  return (
    <section
      data-component="official-deepseek"
      className="space-y-3 rounded-lg border p-4"
    >
      <div className="flex flex-wrap items-center gap-2">
        <KeyRound
          className="size-4 shrink-0 text-muted-foreground"
          aria-hidden="true"
        />
        <h3 className="text-sm font-medium">
          {t('dsh.officialDeepseek.title')}
        </h3>
        <Badge
          variant={configured ? 'secondary' : 'outline'}
          className="text-xs"
        >
          {configured
            ? t('dsh.officialDeepseek.configured')
            : t('dsh.officialDeepseek.notConfigured')}
        </Badge>
      </div>

      <p className="text-sm text-muted-foreground">
        {t('dsh.officialDeepseek.description')}
      </p>
      <p className="text-xs text-muted-foreground">
        {t('dsh.officialDeepseek.loginNote')}
      </p>

      <div className="max-w-md space-y-2">
        <Label htmlFor="dsh-official-deepseek-key">
          {t('dsh.officialDeepseek.apiKey')}
        </Label>
        <SecretInput
          id="dsh-official-deepseek-key"
          value={secretValue}
          onChange={event => setSecretValue(event.target.value)}
          placeholder={
            configured
              ? t('dsh.officialDeepseek.keepExisting')
              : t('dsh.officialDeepseek.placeholder')
          }
        />
      </div>

      <div className="space-y-1">
        <p className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <span>{t('dsh.officialDeepseek.ref', { ref: effectiveRef })}</span>
          {customRef.trim() ? null : (
            <Badge variant="outline" className="text-xs">
              {t('dsh.officialDeepseek.default')}
            </Badge>
          )}
          <Button
            type="button"
            variant="link"
            size="sm"
            className="h-auto p-0 text-xs"
            onClick={() => {
              setRefDraft({
                entryRef,
                custom: expanded ? '' : customRef,
                expanded: !expanded,
              })
            }}
          >
            {expanded
              ? t('dsh.officialDeepseek.useDefault')
              : t('dsh.officialDeepseek.custom')}
          </Button>
        </p>

        {expanded ? (
          <div className="max-w-md space-y-2">
            <Label htmlFor="dsh-official-deepseek-ref">
              {t('dsh.officialDeepseek.refLabel')}
            </Label>
            <Input
              id="dsh-official-deepseek-ref"
              value={customRef}
              onChange={event =>
                setRefDraft({
                  entryRef,
                  custom: event.target.value,
                  expanded: true,
                })
              }
              placeholder={entryRef}
            />
            <p className="text-xs text-muted-foreground">
              {t('dsh.officialDeepseek.refHint')}
            </p>
            {refInvalid ? (
              <p className="text-xs text-amber-600 dark:text-amber-400">
                {t('dsh.officialDeepseek.invalidRef')}
              </p>
            ) : null}
          </div>
        ) : null}
      </div>

      <div className="flex flex-wrap items-center justify-end gap-2">
        {configured ? (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => setClearOpen(true)}
          >
            <Trash2 aria-hidden="true" />
            {t('dsh.officialDeepseek.clear')}
          </Button>
        ) : null}
        <Button
          type="button"
          size="sm"
          disabled={saving || refInvalid || !hasChanges}
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
          {saving
            ? t('dsh.officialDeepseek.saving')
            : t('dsh.officialDeepseek.save')}
        </Button>
      </div>

      <AlertDialog open={clearOpen} onOpenChange={setClearOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('dsh.officialDeepseek.clearTitle')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('dsh.officialDeepseek.clearConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                void handleClear()
              }}
            >
              {t('dsh.officialDeepseek.clear')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  )
}
