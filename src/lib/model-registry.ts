import registryData from './model-registry-data.json'
import type { ReasoningEffort } from './utils'

export type ModelPlatform =
  | 'openai-completions'
  | 'openai-responses'
  | 'anthropic-messages'
  | 'gemini'

/** 单个 effort 级别的编码规则 */
export interface EffortEncoding {
  /** 该 effort 级别对应的 extraArgs JSON 片段 */
  extraArgsFragment: Record<string, unknown>
}

/** 模型推理配置（白名单机制） */
export interface ModelReasoningConfig {
  /** 支持的 effort 级别 */
  efforts: ReasoningEffort[]
  /** 按 provider 区分的编码规则，key 为 effort 值 */
  encoding: Record<string, Record<string, EffortEncoding>>
}

export interface ModelRegistryEntry {
  /** Primary model ID (e.g. "claude-sonnet-4-20250514") */
  id: string
  /** Display name (e.g. "Claude Sonnet 4") */
  name: string
  /** Alternative IDs that map to this model */
  aliases: string[]
  /** Default API platform type */
  platform: ModelPlatform
  /** Context window size in tokens */
  contextWindow: number
  /** Maximum output tokens */
  maxOutputTokens?: number
  /** 推理配置（白名单，未设置则走旧逻辑） */
  reasoningConfig?: ModelReasoningConfig
}

const registry: ModelRegistryEntry[] = registryData as ModelRegistryEntry[]

// Build a lookup map: id/alias -> entry
const lookupMap = new Map<string, ModelRegistryEntry>()
for (const entry of registry) {
  lookupMap.set(entry.id, entry)
  for (const alias of entry.aliases) {
    lookupMap.set(alias, entry)
  }
}

/**
 * Find a model by its ID or any of its aliases.
 * Returns undefined if not found.
 */
export function findModelByIdOrAlias(
  id: string
): ModelRegistryEntry | undefined {
  return lookupMap.get(id)
}

/**
 * Get all registered models, sorted alphabetically by ID.
 */
export function getAllRegistryModels(): ModelRegistryEntry[] {
  return [...registry].sort((a, b) => a.id.localeCompare(b.id))
}

/**
 * 获取模型的推理配置（白名单优先）。
 * 在 registry 中存在 reasoningConfig 则返回，否则返回 null（走旧逻辑）。
 */
export function getModelReasoningConfig(
  modelId: string
): ModelReasoningConfig | null {
  if (!modelId) return null
  const entry = lookupMap.get(modelId)
  return entry?.reasoningConfig ?? null
}

/**
 * 获取某个模型+provider 支持的可选 effort 列表。
 * 白名单有配置 → 返回白名单 effort 列表；
 * 未配置 → 返回 null，调用方走旧逻辑。
 */
export function getSupportedEfforts(
  modelId: string,
  provider: string
): ReasoningEffort[] | null {
  const config = getModelReasoningConfig(modelId)
  if (!config) return null
  // 检查 provider 是否在 encoding 中有定义
  if (!config.encoding[provider]) return config.efforts
  return config.efforts
}

/**
 * 获取模型+provider+effort 的编码片段。
 * 白名单有配置 → 返回 extraArgsFragment；
 * 未配置 → 返回 null，调用方走旧逻辑。
 */
export function getEffortEncoding(
  modelId: string,
  provider: string,
  effort: string
): Record<string, unknown> | null {
  const config = getModelReasoningConfig(modelId)
  if (!config) return null
  const providerEncoding = config.encoding[provider]
  if (!providerEncoding) return null
  return providerEncoding[effort]?.extraArgsFragment ?? null
}
