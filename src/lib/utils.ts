import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function containsRegexSpecialChars(value: string): boolean {
  return /[[\](){}^$.*+?|\\]/.test(value)
}

export function getDefaultMaxOutputTokens(modelId: string): number {
  return modelId.startsWith('claude-') ? 64000 : 16384
}
