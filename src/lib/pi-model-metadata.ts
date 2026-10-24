import {
  findModelByIdOrAlias,
  type ModelRegistryEntry,
} from '@/lib/model-registry'
import type { PiCompatConfig, PiModel } from '@/lib/bindings'

/**
 * Compat flags derived from registry metadata. Models whose Anthropic
 * encoding profile is `anthropic-adaptive` (e.g. claude-opus-5-5) require
 * thinking.type: adaptive + output_config.effort, which pi only sends for
 * custom providers when compat.forceAdaptiveThinking is set.
 */
function piCompatForEntry(entry: ModelRegistryEntry): PiCompatConfig | null {
  return entry.reasoningConfig?.profiles?.anthropic === 'anthropic-adaptive'
    ? { forceAdaptiveThinking: true }
    : null
}

/** Merge registry-derived compat under existing user-defined compat. */
function mergePiCompat(
  existing: PiCompatConfig | null | undefined,
  derived: PiCompatConfig | null
): PiCompatConfig | null {
  if (!derived) return existing ?? null
  return { ...derived, ...existing }
}

export function createPiModelFromRegistry(entry: ModelRegistryEntry): PiModel {
  return {
    id: entry.id,
    name: entry.name,
    api: null,
    reasoning: entry.reasoning,
    input: entry.input,
    thinkingLevelMap: entry.thinkingLevelMap
      ? { ...entry.thinkingLevelMap }
      : null,
    contextWindow: entry.contextWindow,
    maxTokens: entry.maxOutputTokens,
    cost: null,
    compat: piCompatForEntry(entry),
  }
}

export function enrichPiModelFromRegistry(model: PiModel): PiModel {
  const entry = findModelByIdOrAlias(model.id)
  if (!entry) return model

  return {
    ...model,
    name: entry.name,
    reasoning: entry.reasoning,
    input: entry.input,
    thinkingLevelMap: entry.thinkingLevelMap
      ? { ...entry.thinkingLevelMap }
      : null,
    contextWindow: entry.contextWindow,
    maxTokens: entry.maxOutputTokens ?? model.maxTokens,
    compat: mergePiCompat(model.compat, piCompatForEntry(entry)),
  }
}
