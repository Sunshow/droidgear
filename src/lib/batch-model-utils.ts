import type { CustomModel, Provider } from '@/lib/bindings'
import {
  containsRegexSpecialChars,
  getDefaultMaxOutputTokens,
  hasOfficialModelNamePrefix,
} from '@/lib/utils'

export interface BatchModelConfig {
  alias: string
  provider: Provider
  maxTokens?: number
  noImageSupport?: boolean
}

export function buildModelsFromBatch(
  selectedModels: Map<string, BatchModelConfig>,
  baseUrl: string,
  apiKey: string,
  prefix: string,
  suffix: string,
  batchMaxTokens: string,
  batchNoImageSupport: boolean,
  existingModels: CustomModel[]
): CustomModel[] {
  const models: CustomModel[] = []

  for (const [modelId, config] of selectedModels) {
    if (existingModels.some(m => m.model === modelId && m.apiKey === apiKey)) {
      continue
    }

    let displayName = modelId
    if (config.alias) {
      displayName = config.alias
    } else if (prefix || suffix) {
      displayName = `${prefix}${modelId}${suffix}`
    }

    let maxOutputTokens: number | undefined
    if (config.maxTokens !== undefined) {
      maxOutputTokens = config.maxTokens
    } else if (batchMaxTokens) {
      maxOutputTokens = parseInt(batchMaxTokens)
    } else {
      maxOutputTokens = getDefaultMaxOutputTokens(modelId)
    }

    let noImageSupport: boolean | undefined
    if (config.noImageSupport !== undefined) {
      noImageSupport = config.noImageSupport
    } else if (batchNoImageSupport) {
      noImageSupport = true
    }

    models.push({
      model: modelId,
      baseUrl,
      apiKey,
      provider: config.provider,
      displayName,
      maxOutputTokens,
      noImageSupport: noImageSupport || undefined,
    })
  }

  return models
}

export function isBatchValid(
  selectedModels: Map<string, BatchModelConfig>,
  prefix: string,
  suffix: string
): boolean {
  if (selectedModels.size === 0) return false
  if (containsRegexSpecialChars(prefix)) return false
  if (containsRegexSpecialChars(suffix)) return false

  return Array.from(selectedModels.values()).every(
    config =>
      !containsRegexSpecialChars(config.alias) &&
      !hasOfficialModelNamePrefix(config.alias)
  )
}
