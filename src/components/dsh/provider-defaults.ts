import type { DshConfigSnapshot, DshModel } from '@/lib/bindings'
import { findModelByIdOrAlias, getSupportedEfforts } from '@/lib/model-registry'
import { findEntry, isJsonObject } from './dsh-config-utils'

/** Thinking levels accepted by the pi-ai route's `reasoning` field. */
export const PROVIDER_REASONING_LEVELS = [
  'off',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
] as const

export type ProviderReasoningLevel = (typeof PROVIDER_REASONING_LEVELS)[number]

/** Whether a stored value is one of the route's accepted thinking levels. */
export function isProviderReasoningLevel(
  value: string | null | undefined
): value is ProviderReasoningLevel {
  return (
    value !== null &&
    value !== undefined &&
    (PROVIDER_REASONING_LEVELS as readonly string[]).includes(value)
  )
}

/** One model row's identity plus the metadata fetched from its provider. */
export interface ReasoningModel {
  id: string
  base?: DshModel | null
}

/** The DSH default model (`agent-default-model` entry config). */
export interface DshDefaultModel {
  provider: string
  model: string
  reasoningEffort: string | null
}

/**
 * The registry spells the disabled level `none`; the pi-ai route uses `off`.
 */
function normalizeLevel(level: string): string {
  return level === 'none' ? 'off' : level
}

/**
 * Thinking levels one model is known to support, or `null` when neither the
 * fetched metadata nor the built-in registry can tell.
 *
 * Capability priority: the provider's own `reasoningEfforts` map, then the
 * registry's reasoning config, then its thinking-level map.
 */
export function supportedReasoningLevels(
  model: ReasoningModel
): string[] | null {
  const fetched = model.base?.reasoningEfforts
  if (fetched && Object.keys(fetched).length > 0) {
    return Object.keys(fetched).map(normalizeLevel)
  }

  const id = model.id.trim()
  if (!id) return null

  const registryEfforts = getSupportedEfforts(id, 'openai')
  if (registryEfforts && registryEfforts.length > 0) {
    return registryEfforts.map(normalizeLevel)
  }

  const thinkingLevels = findModelByIdOrAlias(id)?.thinkingLevelMap
  if (thinkingLevels && Object.keys(thinkingLevels).length > 0) {
    return Object.keys(thinkingLevels)
  }

  return null
}

/**
 * Suggest a route-level default thinking level.
 *
 * `high` is suggested only when every model row is known and supports it;
 * rows whose capabilities are unknown block the suggestion, because the
 * route default applies to all of them.
 */
export function suggestDefaultReasoning(
  models: ReasoningModel[]
): ProviderReasoningLevel | null {
  if (models.length === 0) return null
  for (const model of models) {
    const levels = supportedReasoningLevels(model)
    if (levels === null || !levels.includes('high')) return null
  }
  return 'high'
}

/**
 * Model ids known to reject a thinking level. Rows with unknown capabilities
 * are ignored: the warning only reports what is actually known.
 */
export function modelsRejectingLevel(
  models: ReasoningModel[],
  level: string
): string[] {
  if (!level) return []
  return models
    .filter(model => {
      const levels = supportedReasoningLevels(model)
      return levels !== null && !levels.includes(level)
    })
    .map(model => model.id.trim() || '(unnamed)')
}

/** Read the configured DSH default model from a composed snapshot. */
export function readDefaultModel(
  snapshot: DshConfigSnapshot | null
): DshDefaultModel | null {
  const entry = findEntry(snapshot, 'agent-default-model')
  if (!entry) return null

  const config = isJsonObject(entry.config) ? entry.config : null
  const readValue = (name: string): string | null => {
    const fromConfig = config?.[name]
    if (typeof fromConfig === 'string') return fromConfig
    const field = entry.fields.find(item => item.name === name)
    return typeof field?.value === 'string' ? field.value : null
  }

  const provider = readValue('provider')
  const model = readValue('model')
  if (!provider || !model) return null

  return {
    provider,
    model,
    reasoningEffort: readValue('reasoningEffort'),
  }
}
