import registryData from './model-registry-data.json'
import type { ReasoningEffort } from './utils'

export type ModelPlatform =
  | 'openai-completions'
  | 'openai-responses'
  | 'anthropic-messages'
  | 'gemini'

export type EffortProvider =
  | 'anthropic'
  | 'openai'
  | 'generic-chat-completion-api'

/** Named encoding profiles expanded at runtime into extraArgs fragments. */
export type EffortEncodingProfile =
  | 'openai-reasoning'
  | 'openai-thinking'
  | 'anthropic-adaptive'
  | 'anthropic-budget'
  | 'anthropic-output-config'

/**
 * Reasoning parameter format for OpenAI-compatible providers.
 * - `reasoning`: `{ reasoning: { effort } }`
 * - `thinking`: `{ thinking: { type: "enabled" }, reasoning_effort }` (and
 *   `{ thinking: { type: "disabled" } }` for the `none` level)
 */
export type EffortFormat = 'reasoning' | 'thinking'

/** 单个 effort 级别的编码规则 */
export interface EffortEncoding {
  /** 该 effort 级别对应的 extraArgs JSON 片段 */
  extraArgsFragment: Record<string, unknown>
}

/** 模型推理配置（白名单机制） */
export interface ModelReasoningConfig {
  /** 支持的 effort 级别 */
  efforts: ReasoningEffort[]
  /**
   * Provider -> named encoding profile.
   * Used when no custom encoding fragment is present for that provider/effort.
   */
  profiles?: Partial<Record<EffortProvider, EffortEncodingProfile>>
  /**
   * Optional full custom fragments (e.g. deepseek / glm).
   * Wins over profiles when present for the same provider+effort.
   */
  encoding?: Record<string, Record<string, EffortEncoding>>
}

export type PiThinkingLevel =
  | 'off'
  | 'minimal'
  | 'low'
  | 'medium'
  | 'high'
  | 'xhigh'
  | 'max'

export type PiThinkingLevelMap = Partial<Record<PiThinkingLevel, string | null>>

export interface ModelRegistryEntry {
  /** Primary model ID (e.g. "claude-sonnet-4-20250514") */
  id: string
  /** Display name (e.g. "Claude Sonnet 4") */
  name: string
  /** Alternative IDs that map to this model */
  aliases: string[]
  /** Default API platform type */
  platform: ModelPlatform
  /** Whether the model supports extended thinking */
  reasoning: boolean
  /** Input modalities supported by Pi model configuration */
  input: ('text' | 'image')[]
  /** Provider-neutral Pi thinking capabilities for custom providers */
  thinkingLevelMap?: PiThinkingLevelMap
  /** Context window size in tokens */
  contextWindow: number
  /** Maximum output tokens */
  maxOutputTokens?: number
  /** Rejects sampling parameters (temperature, top_p, top_k) */
  strictSampling?: boolean
  /** 推理配置（白名单，未设置则走旧逻辑） */
  reasoningConfig?: ModelReasoningConfig
}

const EFFORT_BUDGET_TOKENS: Record<string, number> = {
  low: 4096,
  medium: 8192,
  high: 16384,
  xhigh: 32768,
  max: 32768,
}

function expandProfile(
  profile: EffortEncodingProfile,
  effort: string
): Record<string, unknown> | null {
  switch (profile) {
    case 'openai-reasoning':
      return { reasoning: { effort } }
    case 'openai-thinking':
      return effort === 'none'
        ? { thinking: { type: 'disabled' } }
        : { thinking: { type: 'enabled' }, reasoning_effort: effort }
    case 'anthropic-adaptive':
      return {
        thinking: { type: 'adaptive' },
        output_config: { effort },
      }
    case 'anthropic-budget':
      return {
        thinking: {
          type: 'enabled',
          budget_tokens: EFFORT_BUDGET_TOKENS[effort] ?? 4096,
        },
      }
    case 'anthropic-output-config':
      return {
        thinking: { type: 'enabled' },
        output_config: { effort },
      }
    default:
      return null
  }
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
  _provider: string
): ReasoningEffort[] | null {
  const config = getModelReasoningConfig(modelId)
  if (!config) return null
  return config.efforts
}

const EFFORT_ORDER: ReasoningEffort[] = [
  'none',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
]

/**
 * Clamp an effort value to the highest supported option at or below it.
 * Falls back to the first supported effort when nothing matches.
 */
export function clampEffortToSupported(
  effort: string,
  supported: readonly string[],
  fallback = 'high'
): string {
  if (supported.length === 0) return fallback
  if (supported.includes(effort)) return effort

  const idx = EFFORT_ORDER.indexOf(effort as ReasoningEffort)
  if (idx >= 0) {
    for (let i = idx - 1; i >= 0; i--) {
      const candidate = EFFORT_ORDER[i]
      if (candidate && supported.includes(candidate)) return candidate
    }
  }

  // Prefer a sensible default if present, otherwise first supported value.
  if (supported.includes(fallback)) return fallback
  return supported[0] ?? fallback
}

const OPENAI_LIKE_PROVIDERS: ReadonlySet<string> = new Set([
  'openai',
  'generic-chat-completion-api',
])

/** Whether the provider speaks an OpenAI-compatible chat-completions API. */
export function isOpenAiLikeProvider(provider: string): boolean {
  return OPENAI_LIKE_PROVIDERS.has(provider)
}

/**
 * Expand an OpenAI-compatible effort encoding for the given format, with no
 * model lookup. Used both to override the `openai-reasoning` profile and as the
 * fallback for unregistered OpenAI-compatible models.
 *
 * Returns null when there is nothing to inject (`reasoning` format + `none`).
 */
export function expandOpenAiEffortEncoding(
  effort: string,
  format: EffortFormat
): Record<string, unknown> | null {
  if (format === 'reasoning' && effort === 'none') return null
  return expandProfile(
    format === 'thinking' ? 'openai-thinking' : 'openai-reasoning',
    effort
  )
}

/**
 * Whether the model carries a registry-defined custom encoding for this
 * provider (e.g. deepseek). When true the registry encoding is authoritative
 * and the reasoning parameter format cannot be overridden from the UI.
 */
export function hasCustomEffortEncoding(
  modelId: string,
  provider: string
): boolean {
  const config = getModelReasoningConfig(modelId)
  if (!config) return false
  return config.encoding?.[provider] != null
}

/**
 * 获取模型+provider+effort 的编码片段。
 * 白名单有配置 → 返回 extraArgsFragment；
 * 未配置 → 返回 null，调用方走旧逻辑。
 *
 * Resolution order:
 * 1. custom encoding[provider][effort]
 * 2. expand profiles[provider] (with an optional OpenAI format override)
 * 3. null
 */
export function getEffortEncoding(
  modelId: string,
  provider: string,
  effort: string,
  format?: EffortFormat
): Record<string, unknown> | null {
  const config = getModelReasoningConfig(modelId)
  if (!config) return null

  const custom = config.encoding?.[provider]?.[effort]?.extraArgsFragment
  if (custom) return custom

  const profile = config.profiles?.[provider as EffortProvider]
  if (!profile) return null

  // Let the UI override the OpenAI reasoning parameter format for
  // OpenAI-compatible providers (reasoning.effort vs thinking + reasoning_effort).
  if (
    format &&
    isOpenAiLikeProvider(provider) &&
    profile === 'openai-reasoning'
  ) {
    return expandOpenAiEffortEncoding(effort, format)
  }

  // Profiles don't define a `none` form — only explicit custom encodings
  // (handled above) do.
  if (effort === 'none') return null

  return expandProfile(profile, effort)
}
