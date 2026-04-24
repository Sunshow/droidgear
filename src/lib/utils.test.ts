import { describe, it, expect } from 'vitest'
import {
  effortToBudgetTokens,
  getDefaultMaxOutputTokens,
  isAnthropicAdaptiveThinkingModel,
  isOpus47,
  supportsMaxEffort,
  supportsXhighEffort,
} from './utils'

describe('isOpus47', () => {
  it('matches both dotted and dashed spellings', () => {
    expect(isOpus47('claude-opus-4.7')).toBe(true)
    expect(isOpus47('claude-opus-4-7')).toBe(true)
    expect(isOpus47('claude-opus-4-7-1m')).toBe(true)
  })

  it('does not match other opus versions', () => {
    expect(isOpus47('claude-opus-4')).toBe(false)
    expect(isOpus47('claude-opus-4.6')).toBe(false)
    expect(isOpus47('claude-opus-4.5')).toBe(false)
  })
})

describe('isAnthropicAdaptiveThinkingModel', () => {
  it('matches Opus 4.7 / 4.6 and Sonnet 4.6', () => {
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.7')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4-7')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.6')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-sonnet-4.6')).toBe(true)
  })

  it('rejects older claude models', () => {
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.5')).toBe(false)
    expect(isAnthropicAdaptiveThinkingModel('claude-sonnet-4.5')).toBe(false)
    expect(isAnthropicAdaptiveThinkingModel('claude-haiku-4.5')).toBe(false)
  })
})

describe('supportsMaxEffort', () => {
  it('applies to all anthropic adaptive thinking models', () => {
    expect(supportsMaxEffort('claude-opus-4.7')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4-7')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4.6')).toBe(true)
    expect(supportsMaxEffort('claude-sonnet-4.6')).toBe(true)
  })

  it('does not apply to legacy claude or openai models', () => {
    expect(supportsMaxEffort('claude-opus-4.5')).toBe(false)
    expect(supportsMaxEffort('claude-sonnet-4.5')).toBe(false)
    expect(supportsMaxEffort('gpt-5.2')).toBe(false)
  })
})

describe('supportsXhighEffort', () => {
  it('allows xhigh on Opus 4.7 and openai reasoning models', () => {
    expect(supportsXhighEffort('claude-opus-4.7')).toBe(true)
    expect(supportsXhighEffort('gpt-5.2')).toBe(true)
    expect(supportsXhighEffort('o3-mini')).toBe(true)
  })

  it('rejects xhigh on Opus 4.6 and Sonnet 4.6 (adaptive non-4.7)', () => {
    expect(supportsXhighEffort('claude-opus-4.6')).toBe(false)
    expect(supportsXhighEffort('claude-sonnet-4.6')).toBe(false)
  })

  it('allows xhigh on legacy claude models via budget_tokens mapping', () => {
    expect(supportsXhighEffort('claude-opus-4.5')).toBe(true)
    expect(supportsXhighEffort('claude-sonnet-4.5')).toBe(true)
  })

  it('is permissive for unknown/empty IDs', () => {
    expect(supportsXhighEffort('')).toBe(true)
  })
})

describe('getDefaultMaxOutputTokens', () => {
  it('uses 64k for Opus 4.7 at xhigh/max effort', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'xhigh')).toBe(64000)
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'max')).toBe(64000)
    expect(getDefaultMaxOutputTokens('claude-opus-4-7', 'max')).toBe(64000)
  })

  it('uses 32k for Opus 4.7 at lower efforts', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.7')).toBe(32000)
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'none')).toBe(32000)
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'high')).toBe(32000)
  })

  it('uses 64k for other claude models', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.6')).toBe(64000)
    expect(getDefaultMaxOutputTokens('claude-sonnet-4.5')).toBe(64000)
  })

  it('uses 16384 for non-claude models', () => {
    expect(getDefaultMaxOutputTokens('gpt-5.2')).toBe(16384)
    expect(getDefaultMaxOutputTokens('gemini-2.5-pro')).toBe(16384)
  })
})

describe('effortToBudgetTokens', () => {
  it('maps known efforts to budget sizes', () => {
    expect(effortToBudgetTokens('low')).toBe(4096)
    expect(effortToBudgetTokens('medium')).toBe(8192)
    expect(effortToBudgetTokens('high')).toBe(16384)
    expect(effortToBudgetTokens('xhigh')).toBe(32768)
    expect(effortToBudgetTokens('max')).toBe(32768)
  })

  it('falls back to a safe minimum for unknown values', () => {
    expect(effortToBudgetTokens('none')).toBe(4096)
    expect(effortToBudgetTokens('')).toBe(4096)
  })
})
