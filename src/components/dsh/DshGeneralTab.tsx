import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { useDshStore } from '@/store/dsh-store'
import type { DshConfigEntryView } from '@/lib/bindings'
import { EntrySettingsCard } from './config/EntrySettingsCard'
import { findEntry } from './dsh-config-utils'
import { useOptionLabel } from './dsh-option-labels'

const GENERAL_ENTRIES = [
  'locale',
  'ui-theme',
  'ui-chat',
  'ui-conversation',
  'ui-settings',
  'session-log-deepseek',
] as const

interface FieldSpec {
  fields: readonly string[]
  labels: Record<string, ReactNode>
}

/** General settings tab: language, appearance, chat, and developer tools. */
export function DshGeneralTab() {
  const { t } = useTranslation()
  const snapshot = useDshStore(state => state.snapshot)
  const isLoading = useDshStore(state => state.isLoadingSnapshot)
  const saveEntry = useDshStore(state => state.saveEntry)
  const savingEntryId = useDshStore(state => state.savingEntryId)
  const optionLabel = useOptionLabel()

  const fieldSpec = (entryId: string): FieldSpec => {
    switch (entryId) {
      case 'locale':
        return {
          fields: ['preference'],
          labels: { preference: t('dsh.field.label.locale.preference') },
        }
      case 'ui-theme':
        return {
          fields: [
            'preference',
            'fontSize',
            'codeFontSize',
            'terminalFontSize',
            'textFontFamily',
            'codeFontFamily',
            'terminalFontFamily',
          ],
          labels: {
            preference: t('dsh.field.label.ui-theme.preference'),
            fontSize: t('dsh.field.label.ui-theme.fontSize'),
            codeFontSize: t('dsh.field.label.ui-theme.codeFontSize'),
            terminalFontSize: t('dsh.field.label.ui-theme.terminalFontSize'),
            textFontFamily: t('dsh.field.label.ui-theme.textFontFamily'),
            codeFontFamily: t('dsh.field.label.ui-theme.codeFontFamily'),
            terminalFontFamily: t(
              'dsh.field.label.ui-theme.terminalFontFamily'
            ),
          },
        }
      case 'ui-chat':
        return {
          fields: [
            'transcriptView',
            'collapseTiming',
            'performanceUsage',
            'linkOpening',
          ],
          labels: {
            transcriptView: t('dsh.field.label.ui-chat.transcriptView'),
            collapseTiming: t('dsh.field.label.ui-chat.collapseTiming'),
            performanceUsage: t('dsh.field.label.ui-chat.performanceUsage'),
            linkOpening: t('dsh.field.label.ui-chat.linkOpening'),
          },
        }
      case 'ui-conversation':
        return {
          fields: ['busyEnter'],
          labels: {
            busyEnter: t('dsh.field.label.ui-conversation.busyEnter'),
          },
        }
      case 'ui-settings':
        return {
          fields: ['enabled'],
          labels: { enabled: t('dsh.field.label.ui-settings.enabled') },
        }
      case 'session-log-deepseek':
        return {
          fields: ['enabled'],
          labels: {
            enabled: t('dsh.field.label.session-log-deepseek.enabled'),
          },
        }
      default:
        return { fields: [], labels: {} }
    }
  }

  if (!snapshot) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        {isLoading ? t('dsh.config.loading') : t('dsh.config.noSnapshot')}
      </p>
    )
  }

  const entries = GENERAL_ENTRIES.map(entryId =>
    findEntry(snapshot, entryId)
  ).filter((entry): entry is DshConfigEntryView => entry !== undefined)

  if (entries.length === 0) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        {t('dsh.config.noEntries')}
      </p>
    )
  }

  return (
    <div className="space-y-4">
      {entries.map(entry => {
        const spec = fieldSpec(entry.id)
        return (
          <EntrySettingsCard
            key={entry.id}
            entry={entry}
            fields={spec.fields}
            labels={spec.labels}
            fieldOverrides={
              entry.id === 'locale'
                ? {
                    preference: {
                      kind: 'enum',
                      options: [
                        { value: 'zh', label: optionLabel('zh') },
                        { value: 'en', label: optionLabel('en') },
                      ],
                    },
                  }
                : undefined
            }
            editableTiers={['A', 'B', 'C']}
            optionLabel={optionLabel}
            saving={savingEntryId === entry.id}
            onSave={config => saveEntry(entry.id, config)}
          />
        )
      })}
    </div>
  )
}
