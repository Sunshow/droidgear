import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { useDshStore } from '@/store/dsh-store'
import type { DshConfigEntryView } from '@/lib/bindings'
import { EntrySettingsCard } from './config/EntrySettingsCard'
import { SettingsSection } from './config/SettingsSection'
import { findEntry } from './dsh-config-utils'
import { useOptionLabel } from './dsh-option-labels'

const NETWORK_ENTRIES = [
  'web',
  'web-search-deepseek',
  'web-deepseek-api',
  'web-fetch-http',
] as const

interface FieldSpec {
  fields: readonly string[]
  labels: Record<string, ReactNode>
  hints?: Record<string, ReactNode>
}

/** Network tab: search, fetch, and proxy guidance. */
export function DshNetworkTab() {
  const { t } = useTranslation()
  const snapshot = useDshStore(state => state.snapshot)
  const isLoading = useDshStore(state => state.isLoadingSnapshot)
  const saveEntry = useDshStore(state => state.saveEntry)
  const savingEntryId = useDshStore(state => state.savingEntryId)
  const optionLabel = useOptionLabel()

  const fieldSpec = (entryId: string): FieldSpec => {
    switch (entryId) {
      case 'web':
        return {
          fields: ['searchProvider', 'fetchProvider'],
          labels: {
            searchProvider: t('dsh.field.label.web.searchProvider'),
            fetchProvider: t('dsh.field.label.web.fetchProvider'),
          },
        }
      case 'web-search-deepseek':
        return {
          fields: ['apiKeyEnv', 'baseURL', 'model', 'maxUses', 'apiKey'],
          labels: {
            apiKeyEnv: t('dsh.field.label.web-search-deepseek.apiKeyEnv'),
            baseURL: t('dsh.field.label.web-search-deepseek.baseURL'),
            model: t('dsh.field.label.web-search-deepseek.model'),
            maxUses: t('dsh.field.label.web-search-deepseek.maxUses'),
            apiKey: t('dsh.field.label.web-search-deepseek.apiKey'),
          },
          hints: {
            apiKey: t('dsh.field.hint.web-search-deepseek.apiKey'),
          },
        }
      case 'web-deepseek-api':
        return {
          fields: [
            'apiKeyEnv',
            'baseURL',
            'count',
            'sendOriginalQuestion',
            'apiKey',
          ],
          labels: {
            apiKeyEnv: t('dsh.field.label.web-deepseek-api.apiKeyEnv'),
            baseURL: t('dsh.field.label.web-deepseek-api.baseURL'),
            count: t('dsh.field.label.web-deepseek-api.count'),
            sendOriginalQuestion: t(
              'dsh.field.label.web-deepseek-api.sendOriginalQuestion'
            ),
            apiKey: t('dsh.field.label.web-deepseek-api.apiKey'),
          },
          hints: {
            apiKey: t('dsh.field.hint.web-deepseek-api.apiKey'),
          },
        }
      case 'web-fetch-http':
        return {
          fields: ['timeoutMs', 'maxRedirects', 'userAgent'],
          labels: {
            timeoutMs: t('dsh.field.label.web-fetch-http.timeoutMs'),
            maxRedirects: t('dsh.field.label.web-fetch-http.maxRedirects'),
            userAgent: t('dsh.field.label.web-fetch-http.userAgent'),
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

  const entries = NETWORK_ENTRIES.map(entryId =>
    findEntry(snapshot, entryId)
  ).filter((entry): entry is DshConfigEntryView => entry !== undefined)

  return (
    <div className="space-y-4">
      {entries.length === 0 ? (
        <p className="py-8 text-center text-sm text-muted-foreground">
          {t('dsh.config.noEntries')}
        </p>
      ) : (
        entries.map(entry => {
          const spec = fieldSpec(entry.id)
          return (
            <EntrySettingsCard
              key={entry.id}
              entry={entry}
              fields={spec.fields}
              labels={spec.labels}
              hints={spec.hints}
              optionLabel={optionLabel}
              saving={savingEntryId === entry.id}
              onSave={config => saveEntry(entry.id, config)}
            />
          )
        })
      )}

      <SettingsSection
        title={t('dsh.network.proxy.title')}
        description={t('dsh.network.proxy.description')}
      >
        <p className="text-sm text-muted-foreground">
          {t('dsh.network.proxy.envHint')}
        </p>
      </SettingsSection>
    </div>
  )
}
