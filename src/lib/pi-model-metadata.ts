import {
  findModelByIdOrAlias,
  type ModelRegistryEntry,
} from '@/lib/model-registry'
import type { PiModel } from '@/lib/bindings'

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
    compat: null,
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
  }
}
