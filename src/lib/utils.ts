import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { findModelByIdOrAlias, getModelReasoningConfig } from './model-registry'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * Trim leading/trailing whitespace. Empty or whitespace-only input becomes
 * null, so callers can store the result directly in optional fields.
 */
export function trimToNull(value: string | null | undefined): string | null {
  const trimmed = value?.trim() ?? ''
  return trimmed || null
}

export function containsRegexSpecialChars(value: string): boolean {
  return /[[\](){}^$*+?|\\]/.test(value)
}

export type ReasoningEffort =
  | 'none'
  | 'low'
  | 'medium'
  | 'high'
  | 'xhigh'
  | 'max'

function normalizeModelId(modelId: string): string {
  return modelId.toLowerCase().replace(/[-_]/g, '.')
}

// Models that reject sampling parameters (temperature, top_p, top_k).
// Registry-driven: entries flagged strictSampling opt out of sampling params.
export function isStrictSamplingModel(modelId: string): boolean {
  return findModelByIdOrAlias(modelId)?.strictSampling ?? false
}

export function isAnthropicAdaptiveThinkingModel(modelId: string): boolean {
  return (
    findModelByIdOrAlias(modelId)?.reasoningConfig?.profiles?.anthropic ===
    'anthropic-adaptive'
  )
}

export function isRecognizedClaudeModelId(modelId: string): boolean {
  const trimmed = modelId.trim()
  if (!trimmed) return false
  return normalizeModelId(trimmed).startsWith('claude.')
}

export function hasOpaqueClaudeModelId(
  modelId: string | null | undefined
): boolean {
  if (!modelId?.trim()) return false
  return !isRecognizedClaudeModelId(modelId)
}

// Priority: registry reasoningConfig.efforts → pattern matching for
// unregistered model IDs only.
export function supportsMaxEffort(modelId: string): boolean {
  if (!modelId) return true
  const config = getModelReasoningConfig(modelId)
  if (config) return config.efforts.includes('max')
  const n = normalizeModelId(modelId)
  return n.startsWith('claude.')
}

// Priority: registry reasoningConfig.efforts → pattern matching for
// unregistered model IDs only. Claude xhigh is narrow without a registry hit;
// GPT-5 / o-series still accept it via reasoning.effort.
export function supportsXhighEffort(modelId: string): boolean {
  if (!modelId) return true
  const config = getModelReasoningConfig(modelId)
  if (config) return config.efforts.includes('xhigh')
  const n = normalizeModelId(modelId)
  return (
    n.startsWith('gpt.5') ||
    n.startsWith('o1') ||
    n.startsWith('o3') ||
    n.startsWith('o4')
  )
}

export function getDefaultMaxOutputTokens(modelId: string): number {
  // Prefer registry values
  const entry = findModelByIdOrAlias(modelId)
  if (entry?.maxOutputTokens) {
    return entry.maxOutputTokens
  }
  // Generic fallback for unknown models
  return modelId.startsWith('claude-') ? 64000 : 16384
}

export function effortToBudgetTokens(effort: string): number {
  switch (effort) {
    case 'low':
      return 4096
    case 'medium':
      return 8192
    case 'high':
      return 16384
    case 'xhigh':
      return 32768
    case 'max':
      return 32768
    default:
      return 4096
  }
}

export const DROID_OFFICIAL_MODEL_NAMES = [
  'GPT-5.1',
  'GPT-5.1-Codex',
  'GPT-5.1-Codex-Max',
  'GPT-5.2',
  'GPT-5.3-Codex',
  'GPT-5.5',
  'Sonnet 4.5',
  'Sonnet 4.6',
  'Sonnet 5',
  'Opus 4.5',
  'Opus 4.6',
  'Opus 4.6 Fast Mode',
  'Opus 4.7',
  'Opus 4.8',
  'Opus 5',
  'Haiku 4.5',
  'Gemini 3 Pro',
  'Gemini 3 Flash',
  'Gemini 3.1 Pro',
  'GLM-4.6',
  'GLM-4.7',
  'GLM-5.1',
  'GLM-5.2',
  'Kimi K2.6',
  'Kimi K2.7 Code',
  'Kimi K3',
  'DeepSeek V4 Pro',
  'MiniMax M2.7',
]

export function isOfficialModelName(value: string): boolean {
  const trimmed = value.trim()
  return DROID_OFFICIAL_MODEL_NAMES.some(
    name => name.toLowerCase() === trimmed.toLowerCase()
  )
}

const PREFIX_SEPARATORS = /^\s/

export function hasOfficialModelNamePrefix(value: string): boolean {
  const trimmed = value.trim().toLowerCase()
  return DROID_OFFICIAL_MODEL_NAMES.some(name => {
    const nameLower = name.toLowerCase()
    if (trimmed === nameLower) return true
    if (trimmed.startsWith(nameLower)) {
      const suffix = trimmed.slice(nameLower.length)
      return PREFIX_SEPARATORS.test(suffix)
    }
    return false
  })
}
