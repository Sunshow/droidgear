import { describe, it, expect } from 'vitest'
import {
  effortToBudgetTokens,
  getDefaultMaxOutputTokens,
  hasOpaqueClaudeModelId,
  isAnthropicAdaptiveThinkingModel,
  isRecognizedClaudeModelId,
  isStrictSamplingModel,
  supportsMaxEffort,
  supportsXhighEffort,
  trimToNull,
} from './utils'

describe('isStrictSamplingModel', () => {
  it('covers Opus 4.7, 4.8, 5 and Fable 5', () => {
    expect(isStrictSamplingModel('claude-opus-4.7')).toBe(true)
    expect(isStrictSamplingModel('claude-opus-4.8')).toBe(true)
    expect(isStrictSamplingModel('claude-opus-5')).toBe(true)
    expect(isStrictSamplingModel('claude-fable-5')).toBe(true)
  })

  it('flags GPT-5/o-series and Kimi models that reject sampling params', () => {
    expect(isStrictSamplingModel('gpt-5.2')).toBe(true)
    expect(isStrictSamplingModel('gpt-5')).toBe(true)
    expect(isStrictSamplingModel('o3-mini')).toBe(true)
    expect(isStrictSamplingModel('kimi-k2.5')).toBe(true)
    expect(isStrictSamplingModel('kimi-k2.7-code')).toBe(true)
    expect(isStrictSamplingModel('claude-sonnet-5')).toBe(true)
    expect(isStrictSamplingModel('kimi-k3')).toBe(true)
  })

  it('does not flag other models', () => {
    expect(isStrictSamplingModel('claude-opus-4.6')).toBe(false)
    expect(isStrictSamplingModel('claude-sonnet-4.6')).toBe(false)
    expect(isStrictSamplingModel('gpt-5.3-chat-latest')).toBe(false)
    expect(isStrictSamplingModel('claude-opus-4.7-custom-deploy')).toBe(false)
    expect(isStrictSamplingModel('grok-4.6')).toBe(false)
    expect(isStrictSamplingModel('gemini-3.6-flash')).toBe(false)
    expect(isStrictSamplingModel('gemini-3.7-flash')).toBe(false)
  })
})

describe('isAnthropicAdaptiveThinkingModel', () => {
  it('matches Opus 4.6 / 4.7 / 4.8 / 5, Sonnet 4.6 / 5, and Fable 5', () => {
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.7')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4-7')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.8')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4-8')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-5')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.6')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-sonnet-4.6')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-sonnet-5')).toBe(true)
    expect(isAnthropicAdaptiveThinkingModel('claude-fable-5')).toBe(true)
  })

  it('rejects older claude models and unregistered IDs', () => {
    expect(isAnthropicAdaptiveThinkingModel('claude-opus-4.5')).toBe(false)
    expect(isAnthropicAdaptiveThinkingModel('claude-sonnet-4.5')).toBe(false)
    expect(isAnthropicAdaptiveThinkingModel('claude-haiku-4.5')).toBe(false)
    expect(
      isAnthropicAdaptiveThinkingModel('claude-opus-4.7-custom-deploy')
    ).toBe(false)
  })
})

describe('supportsMaxEffort', () => {
  it('applies to all claude- models', () => {
    expect(supportsMaxEffort('claude-opus-4.7')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4-7')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4.8')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4-8')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4.6')).toBe(true)
    expect(supportsMaxEffort('claude-sonnet-4.6')).toBe(true)
    expect(supportsMaxEffort('claude-opus-4.5')).toBe(true)
    expect(supportsMaxEffort('claude-sonnet-4.5')).toBe(true)
    expect(supportsMaxEffort('claude-haiku-4.5')).toBe(true)
    expect(supportsMaxEffort('claude-sonnet-5')).toBe(true)
  })

  it('applies to registry whitelist models with max effort', () => {
    expect(supportsMaxEffort('deepseek-v4-pro')).toBe(true)
    expect(supportsMaxEffort('gpt-5.6')).toBe(true)
    expect(supportsMaxEffort('gpt-5.6-luna')).toBe(true)
    expect(supportsMaxEffort('kimi-k3')).toBe(true)
  })

  it('does not apply to older openai models without max', () => {
    expect(supportsMaxEffort('gpt-5.2')).toBe(false)
    expect(supportsMaxEffort('o3-mini')).toBe(false)
  })
})

describe('supportsXhighEffort', () => {
  it('allows xhigh from registry for Claude and openai reasoning models', () => {
    // All registered Claude models expose full effort list including xhigh.
    expect(supportsXhighEffort('claude-opus-4.7')).toBe(true)
    expect(supportsXhighEffort('claude-opus-4-7')).toBe(true)
    expect(supportsXhighEffort('claude-opus-4.8')).toBe(true)
    expect(supportsXhighEffort('claude-opus-4-8')).toBe(true)
    expect(supportsXhighEffort('claude-sonnet-5')).toBe(true)
    expect(supportsXhighEffort('claude-opus-4.6')).toBe(true)
    expect(supportsXhighEffort('claude-sonnet-4.6')).toBe(true)
    expect(supportsXhighEffort('claude-opus-4.5')).toBe(true)
    expect(supportsXhighEffort('claude-sonnet-4.5')).toBe(true)
    expect(supportsXhighEffort('claude-haiku-4.5')).toBe(true)
    expect(supportsXhighEffort('claude-sonnet-5')).toBe(true)
    expect(supportsXhighEffort('gpt-5.2')).toBe(true)
    expect(supportsXhighEffort('o3-mini')).toBe(true)
    expect(supportsXhighEffort('grok-4.6')).toBe(true)
  })

  it('respects registry whitelist for xhigh', () => {
    // deepseek-v4-pro has whitelist: ["none", "high", "max"] — no xhigh
    expect(supportsXhighEffort('deepseek-v4-pro')).toBe(false)
    // kimi-k3 whitelist: ["none", "low", "high", "max"] — no xhigh
    expect(supportsXhighEffort('kimi-k3')).toBe(false)
  })

  it('rejects xhigh on non-reasoning registry models', () => {
    expect(supportsXhighEffort('gemini-2.5-pro')).toBe(false)
    expect(supportsXhighEffort('grok-4.5')).toBe(false)
  })

  it('does not grant xhigh to unregistered Claude IDs', () => {
    // Pure registry: unregistered IDs are not whitelisted by name.
    expect(supportsXhighEffort('claude-opus-4.7-custom-deploy')).toBe(false)
    expect(supportsXhighEffort('claude-opus-4.5-custom-deploy')).toBe(false)
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
  it('uses registry value for Opus models regardless of effort', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.7')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-opus-4.8')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-opus-5')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-opus-4-8')).toBe(128000)
  })

  it('uses registry values for other claude models', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.6')).toBe(128000)
    expect(getDefaultMaxOutputTokens('claude-sonnet-4.5')).toBe(64000)
    expect(getDefaultMaxOutputTokens('claude-sonnet-5')).toBe(128000)
  })

  it('uses registry values for non-claude models', () => {
    expect(getDefaultMaxOutputTokens('gpt-5.2')).toBe(128000)
    expect(getDefaultMaxOutputTokens('gemini-2.5-pro')).toBe(65536)
    expect(getDefaultMaxOutputTokens('kimi-k3')).toBe(131072)
    expect(getDefaultMaxOutputTokens('grok-4.6')).toBe(500000)
    expect(getDefaultMaxOutputTokens('gemini-3.6-flash')).toBe(65536)
    expect(getDefaultMaxOutputTokens('gemini-3.7-flash')).toBe(65536)
  })

  it('falls back to generic rules for unregistered IDs', () => {
    expect(getDefaultMaxOutputTokens('claude-opus-4.7-custom-deploy')).toBe(
      64000
    )
    expect(getDefaultMaxOutputTokens('gateway-prod-model')).toBe(16384)
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

describe('trimToNull', () => {
  it('trims leading and trailing whitespace', () => {
    expect(trimToNull('  https://api.example.com  ')).toBe(
      'https://api.example.com'
    )
    expect(trimToNull('\tsk-abc123\t')).toBe('sk-abc123')
    expect(trimToNull('\nsk-abc123\r\n')).toBe('sk-abc123')
  })

  it('returns null for empty or whitespace-only input', () => {
    expect(trimToNull('')).toBeNull()
    expect(trimToNull('   ')).toBeNull()
    expect(trimToNull('\t\n ')).toBeNull()
  })

  it('returns null for null and undefined', () => {
    expect(trimToNull(null)).toBeNull()
    expect(trimToNull(undefined)).toBeNull()
  })

  it('keeps non-whitespace content intact', () => {
    expect(trimToNull('sk-abc123')).toBe('sk-abc123')
  })
})
