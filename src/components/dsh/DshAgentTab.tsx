import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { useDshStore } from '@/store/dsh-store'
import type { DshConfigEntryView } from '@/lib/bindings'
import {
  EntrySettingsCard,
  type FieldOverride,
} from './config/EntrySettingsCard'
import {
  collectModelOptions,
  collectProviderOptions,
} from './dsh-agent-candidates'
import { findEntry } from './dsh-config-utils'
import { useOptionLabel } from './dsh-option-labels'

const AGENT_ENTRIES = [
  'agent-default-model',
  'agent-loop',
  'subagent',
  'subagent-model-selection-settings',
  'permission',
  'sandbox-policy',
  'approval',
  'agent-preset-registry',
] as const

interface FieldSpec {
  fields: readonly string[]
  labels: Record<string, ReactNode>
  hints?: Record<string, ReactNode>
}

/** Agent tab: default model, loop, Subagents, and permissions. */
export function DshAgentTab() {
  const { t } = useTranslation()
  const snapshot = useDshStore(state => state.snapshot)
  const isLoading = useDshStore(state => state.isLoadingSnapshot)
  const saveEntry = useDshStore(state => state.saveEntry)
  const savingEntryId = useDshStore(state => state.savingEntryId)
  const optionLabel = useOptionLabel()

  const fieldSpec = (entryId: string): FieldSpec => {
    switch (entryId) {
      case 'agent-default-model':
        return {
          fields: ['provider', 'model', 'reasoningEffort'],
          labels: {
            provider: t('dsh.field.label.agent-default-model.provider'),
            model: t('dsh.field.label.agent-default-model.model'),
            reasoningEffort: t(
              'dsh.field.label.agent-default-model.reasoningEffort'
            ),
          },
        }
      case 'agent-loop':
        return {
          fields: ['maxParallelToolCalls'],
          labels: {
            maxParallelToolCalls: t(
              'dsh.field.label.agent-loop.maxParallelToolCalls'
            ),
          },
        }
      case 'subagent':
        return {
          fields: ['maxDepth', 'maxActiveSubagents'],
          labels: {
            maxDepth: t('dsh.field.label.subagent.maxDepth'),
            maxActiveSubagents: t(
              'dsh.field.label.subagent.maxActiveSubagents'
            ),
          },
          hints: {
            maxDepth: t('dsh.field.hint.subagent.maxDepth'),
            maxActiveSubagents: t('dsh.field.hint.subagent.maxActiveSubagents'),
          },
        }
      case 'subagent-model-selection-settings':
        return {
          fields: ['enabled', 'allowedModels'],
          labels: {
            enabled: t(
              'dsh.field.label.subagent-model-selection-settings.enabled'
            ),
            allowedModels: t(
              'dsh.field.label.subagent-model-selection-settings.allowedModels'
            ),
          },
        }
      case 'permission':
        return {
          fields: ['defaultPreset'],
          labels: {
            defaultPreset: t('dsh.field.label.permission.defaultPreset'),
          },
          hints: {
            defaultPreset: t('dsh.field.hint.permission.defaultPreset'),
          },
        }
      case 'sandbox-policy':
        return {
          fields: ['mode'],
          labels: { mode: t('dsh.field.label.sandbox-policy.mode') },
          hints: { mode: t('dsh.field.hint.sandbox-policy.mode') },
        }
      case 'approval':
        return {
          fields: ['policy'],
          labels: { policy: t('dsh.field.label.approval.policy') },
          hints: { policy: t('dsh.field.hint.approval.policy') },
        }
      case 'agent-preset-registry':
        return {
          fields: ['default', 'selectedDefault'],
          labels: {
            default: t('dsh.field.label.agent-preset-registry.default'),
            selectedDefault: t(
              'dsh.field.label.agent-preset-registry.selectedDefault'
            ),
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

  const entries = AGENT_ENTRIES.map(entryId =>
    findEntry(snapshot, entryId)
  ).filter((entry): entry is DshConfigEntryView => entry !== undefined)

  if (entries.length === 0) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        {t('dsh.config.noEntries')}
      </p>
    )
  }

  const providerOptions = collectProviderOptions(snapshot.entries, optionLabel)
  const modelOptions = collectModelOptions(snapshot.entries)

  const fieldOverrides = (
    entry: DshConfigEntryView
  ): Record<string, FieldOverride> => {
    if (entry.id !== 'agent-default-model') return {}
    const reasoningOptions = ['off', 'low', 'high', 'max'].map(value => ({
      value,
      label: optionLabel(value),
    }))
    return {
      provider: { kind: 'enum', options: providerOptions },
      model: { kind: 'enum', options: modelOptions },
      reasoningEffort: { kind: 'enum', options: reasoningOptions },
    }
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
            hints={spec.hints}
            fieldOverrides={fieldOverrides(entry)}
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
