import { useTranslation } from 'react-i18next'
import { useDshStore } from '@/store/dsh-store'
import { EntrySettingsCard } from './config/EntrySettingsCard'
import { useOptionLabel } from './dsh-option-labels'

const TIER_ORDER: Record<string, number> = { A: 0, B: 1, C: 2 }

/** Advanced tab: every configurable field grouped by entry. */
export function DshAdvancedTab() {
  const { t } = useTranslation()
  const snapshot = useDshStore(state => state.snapshot)
  const isLoading = useDshStore(state => state.isLoadingSnapshot)
  const saveEntry = useDshStore(state => state.saveEntry)
  const toggleEntryDisabled = useDshStore(state => state.toggleEntryDisabled)
  const savingEntryId = useDshStore(state => state.savingEntryId)
  const optionLabel = useOptionLabel()

  if (!snapshot) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        {isLoading ? t('dsh.config.loading') : t('dsh.config.noSnapshot')}
      </p>
    )
  }

  const entries = snapshot.entries
    .filter(entry => entry.fields.length > 0 || entry.overridden)
    .slice()
    .sort((a, b) => {
      const tierDelta = (TIER_ORDER[a.tier] ?? 3) - (TIER_ORDER[b.tier] ?? 3)
      return tierDelta !== 0 ? tierDelta : a.id.localeCompare(b.id)
    })

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        {t('dsh.advanced.description')}
      </p>

      {entries.length === 0 ? (
        <p className="py-8 text-center text-sm text-muted-foreground">
          {t('dsh.config.noEntries')}
        </p>
      ) : (
        entries.map(entry => (
          <EntrySettingsCard
            key={entry.id}
            entry={entry}
            collapsible
            showDisabledToggle
            editableTiers={['A', 'B']}
            optionLabel={optionLabel}
            saving={savingEntryId === entry.id}
            onSave={config => saveEntry(entry.id, config)}
            onToggleDisabled={disabled =>
              toggleEntryDisabled(entry.id, disabled)
            }
          />
        ))
      )}
    </div>
  )
}
