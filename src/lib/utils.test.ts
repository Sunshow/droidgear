import { describe, it, expect } from 'vitest'
import {
  effortToBudgetTokens,
  getDefaultMaxOutputTokens,
  hasOpaqueClaudeModelId,
  isAnthropicAdaptiveThinkingModel,
  isClaudeJupiterV1P,
  isOpus47,
  isRecognizedClaudeModelId,
  isStrictSamplingModel,
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

describe('isClaudeJupiterV1P', () => {
  it('matches dash, dot and underscore spellings', () => {
    expect(isClaudeJupiterV1P('claude-jupiter-v1-p')).toBe(true)
    expect(isClaudeJupiterV1P('claude_jupiter_v1_p')).toBe(true)
    expect(isClaudeJupiterV1P('claude-jupiter-v1.p')).toBe(true)
  })

  it('does not match unrelated models', () => {
    expect(isClaudeJupiterV1P('claude-opus-4.7')).toBe(false)
    expect(isClaudeJupiterV1P('claude-jupiter-v2')).toBe(false)
    expect(isClaudeJupiterV1P('jupiter')).toBe(false)
  })
})

describe('isStrictSamplingModel', () => {
  it('covers Opus 4.7 and Jupiter v1 P', () => {
    expect(isStrictSamplingModel('claude-opus-4.7')).toBe(true)
    expect(isStrictSamplingModel('claude-jupiter-v1-p')).toBe(true)
  })

  it('does not flag other models', () => {
    expect(isStrictSamplingModel('claude-opus-4.6')).toBe(false)
    expect(isStrictSamplingModel('claude-sonnet-4.6')).toBe(false)
    expect(isStrictSamplingModel('gpt-5.2')).toBe(false)
  })
})

describe('isAnthropicAdaptiveThinkingModel', () => {
  it('matches Opus 4.7 / 4.6, Sonnet 4.6 and Jupiter v1 P', () => {
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.7')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4-7')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.6')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-sonnet-4.6')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-jupiter-v1-p')).toBe(true)
  })

  it('rejects older claude models', () => {
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.5')).toBe(false)
    expect(isAnthropicAdaptiveThinkingModel('claude-sonnet-4.5')).toBe(false)
    expect(isAnthropicAdaptiveThinkingModel('claude-haiku-4.5')).toBe(false)
  })
})

describe('supportsMaxEffort', () => {
  it('applies to all claude- models', () => {
    expect(supportsMaxEffort('claude-opus-4.7')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4-7')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4.6')).toBe(true)
    expect(supportsMaxEffort('claude-sonnet-4.6')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4.5')).toBe(true)
    expect(supportsMaxEffort('claude-sonnet-4.5')).toBe(true)
    expect(supportsMaxEffort('claude-haiku-4.5')).toBe(true)
    expect(supportsMaxEffort('claude-jupiter-v1-p')).toBe(true)
  })

  it('applies to registry whitelist models with max effort', () => {
    expect(supportsMaxEffort('deepseek-v4-pro')).toBe(true)
  })

  it('does not apply to openai models', () => {
    expect(supportsMaxEffort('gpt-5.2')).toBe(false)
    expect(supportsMaxEffort('o3-mini')).toBe(false)
  })
})

describe('supportsXhighEffort', () => {
  it('allows xhigh on Opus 4.7, Jupiter v1 P and openai reasoning models', () => {
    expect(supportsXhighEffort('claude-opus-4.7')).toBe(true)
    expect(supportsXhighEffort('claude-opus-4-7')).toBe(true)
    expect(supportsXhighEffort('claude-jupiter-v1-p')).toBe(true)
    expect(supportsXhighEffort('gpt-5.2')).toBe(true)
    expect(supportsXhighEffort('o3-mini')).toBe(true)
  })

  it('respects registry whitelist for xhigh', () => {
    // deepseek-v4-pro has whitelist: ["none", "high", "max"] — no xhigh
    expect(supportsXhighEffort('deepseek-v4-pro')).toBe(false)
  })

  it('rejects xhigh on other claude models and non-reasoning models', () => {
    expect(supportsXhighEffort('claude-opus-4.6')).toBe(false)
    expect(supportsXhighEffort('claude-sonnet-4.6')).toBe(false)
    expect(supportsXhighEffort('claude-opus-4.5')).toBe(false)
    expect(supportsXhighEffort('claude-sonnet-4.5')).toBe(false)
    expect(supportsXhighEffort('claude-haiku-4.5')).toBe(false)
    expect(supportsXhighEffort('gemini-2.5-pro')).toBe(false)
  })

  it('is permissive for unknown/empty IDs', () => {
    expect(supportsXhighEffort('')).toBe(true)
  })
})

describe('Claude model id helpers', () => {
  it('recognizes official claude model ids', () => {
    expect(isRecognizedClaudeModelId('claude-sonnet-4-5')).toBe(true)
    expect(isRecognizedClaudeModelId('claude_opus_4_7')).toBe(true)
    expect(isRecognizedClaudeModelId(' claude-haiku-4-5 ')).toBe(true)
  })

  it('treats custom deployment names as opaque', () => {
    expect(hasOpaqueClaudeModelId('gateway-prod-model')).toBe(true)
    expect(hasOpaqueClaudeModelId('anthropic/claude-sonnet-4-5')).toBe(true)
    expect(hasOpaqueClaudeModelId('')).toBe(false)
    expect(hasOpaqueClaudeModelId(null)).toBe(false)
    expect(hasOpaqueClaudeModelId('claude-sonnet-4-5')).toBe(false)
  })
})

describe('getDefaultMaxOutputTokens', () => {
  it('uses registry value for Opus 4.7 regardless of effort', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'xhigh')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'max')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-opus-4-7', 'max')).toBe(128000)
  })

  it('uses registry value for Opus 4.7 at lower efforts', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.7')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'none')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-opus-4.7', 'high')).toBe(128000)
  })

  it('uses registry values for other claude models', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.6')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-sonnet-4.5')).toBe(64000)
    expect(getDefaultMaxOutputTokens('claude-jupiter-v1-p')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-jupiter-v1-p', 'max')).toBe(128000)
  })

  it('uses registry values for non-claude models', () => {
    expect(getDefaultMaxOutputTokens('gpt-5.2')).toBe(128000)
    expect(getDefaultMaxOutputTokens('gemini-2.5-pro')).toBe(64000)
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
