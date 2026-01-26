import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function containsRegexSpecialChars(value: string): boolean {
  return /[[\](){}^$*+?|\\]/.test(value)
}

export function getDefaultMaxOutputTokens(modelId: string): number {
  return modelId.startsWith('claude-') ? 64000 : 16384
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

const PREFIX_SEPARATORS = /^[\s\-_]/

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
