import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { getModelReasoningConfig } from './model-registry'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
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

export function isOpus47(modelId: string): boolean {
  const n = normalizeModelId(modelId)
  return n.includes('opus.4.7')
}

export function isOpus46(modelId: string): boolean {
  const n = normalizeModelId(modelId)
  return n.includes('opus.4.6')
}

export function isSonnet46(modelId: string): boolean {
  const n = normalizeModelId(modelId)
  return n.includes('sonnet.4.6')
}

export function isAnthropicAdaptiveThinkingModel(modelId: string): boolean {
  const n = normalizeModelId(modelId)
  return (
    n.includes('opus.4.7') ||
    n.includes('opus.4.6') ||
    n.includes('sonnet.4.6') ||
    n.includes('mythos')
  )
}

// All Anthropic models support effort via either adaptive-thinking
// (output_config.effort) or budget_tokens (thinking.budget_tokens). Both
// encodings accept the full range of effort values. Let the user decide.
// Priority: registry whitelist → pattern matching fallback.
export function supportsMaxEffort(modelId: string): boolean {
  if (!modelId) return true
  const config = getModelReasoningConfig(modelId)
  if (config) return config.efforts.includes('max')
  const n = normalizeModelId(modelId)
  return n.startsWith('claude.')
}

// All Anthropic models can express xhigh-level reasoning — adaptive models
// use output_config.effort, others map to budget_tokens. Non-Anthropic
// reasoning models (GPT-5, o-series) also accept it via reasoning.effort.
// Priority: registry whitelist → pattern matching fallback.
export function supportsXhighEffort(modelId: string): boolean {
  if (!modelId) return true
  const config = getModelReasoningConfig(modelId)
  if (config) return config.efforts.includes('xhigh')
  const n = normalizeModelId(modelId)
  if (n.startsWith('claude.')) return true
  return (
    n.startsWith('gpt.5') ||
    n.startsWith('o1') ||
    n.startsWith('o3') ||
    n.startsWith('o4')
  )
}

export function getDefaultMaxOutputTokens(
  modelId: string,
  effort?: ReasoningEffort | string | null
): number {
  if (isOpus47(modelId)) {
    if (effort === 'xhigh' || effort === 'max') return 64000
    return 32000
  }
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
  'Sonnet 4.5',
  'Opus 4.5',
  'Haiku 4.5',
  'Gemini 3 Pro',
  'Gemini 3 Flash',
  'GLM-4.6',
  'GLM-4.7',
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
