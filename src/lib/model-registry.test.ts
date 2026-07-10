import { describe, expect, it } from 'vitest'
import {
  clampEffortToSupported,
  findModelByIdOrAlias,
  getAllRegistryModels,
  getEffortEncoding,
  getModelReasoningConfig,
  getSupportedEfforts,
} from './model-registry'

describe('model-registry reasoningConfig coverage', () => {
  it('has reasoningConfig for every registered model', () => {
    const models = getAllRegistryModels()
    expect(models.length).toBeGreaterThan(0)
    for (const model of models) {
      expect(
        model.reasoningConfig,
        `${model.id} missing reasoningConfig`
      ).toBeTruthy()
      expect(model.reasoningConfig?.efforts.length).toBeGreaterThan(0)
    }
  })
})

describe('getSupportedEfforts', () => {
  it('returns full Claude effort list for adaptive Opus', () => {
    expect(getSupportedEfforts('claude-opus-4-8', 'anthropic')).toEqual([
      'none',
      'low',
      'medium',
      'high',
      'xhigh',
      'max',
    ])
  })

  it('returns gpt-5.6 efforts with max', () => {
    expect(getSupportedEfforts('gpt-5.6', 'openai')).toEqual([
      'none',
      'low',
      'medium',
      'high',
      'xhigh',
      'max',
    ])
  })

  it('returns grok-4.5 none-high only', () => {
    expect(getSupportedEfforts('grok-4.5', 'openai')).toEqual([
      'none',
      'low',
      'medium',
      'high',
    ])
  })

  it('returns deepseek whitelist', () => {
    expect(getSupportedEfforts('deepseek-v4-pro', 'openai')).toEqual([
      'none',
      'high',
      'max',
    ])
  })

  it('returns null for unknown model ids', () => {
    expect(getSupportedEfforts('totally-unknown-model', 'openai')).toBeNull()
  })
})

describe('clampEffortToSupported', () => {
  it('keeps supported efforts unchanged', () => {
    expect(
      clampEffortToSupported('high', ['none', 'low', 'medium', 'high'])
    ).toBe('high')
  })

  it('snaps unsupported high efforts down', () => {
    expect(
      clampEffortToSupported('max', ['none', 'low', 'medium', 'high'])
    ).toBe('high')
    expect(clampEffortToSupported('xhigh', ['none', 'high', 'max'])).toBe(
      'high'
    )
  })

  it('falls back to preferred default when present', () => {
    expect(clampEffortToSupported('bogus', ['none', 'high', 'max'])).toBe(
      'high'
    )
  })
})

describe('getEffortEncoding profiles', () => {
  it('expands openai-reasoning profile', () => {
    expect(getEffortEncoding('gpt-5.6', 'openai', 'high')).toEqual({
      reasoning: { effort: 'high' },
    })
  })

  it('expands anthropic-adaptive profile', () => {
    expect(getEffortEncoding('claude-opus-4-8', 'anthropic', 'xhigh')).toEqual({
      thinking: { type: 'adaptive' },
      output_config: { effort: 'xhigh' },
    })
  })

  it('expands anthropic-budget profile with budget_tokens', () => {
    expect(getEffortEncoding('claude-sonnet-4-5', 'anthropic', 'high')).toEqual(
      {
        thinking: { type: 'enabled', budget_tokens: 16384 },
      }
    )
  })

  it('expands anthropic-output-config profile', () => {
    expect(getEffortEncoding('grok-4.5', 'anthropic', 'medium')).toEqual({
      thinking: { type: 'enabled' },
      output_config: { effort: 'medium' },
    })
  })

  it('prefers custom encoding over profiles for deepseek', () => {
    expect(getEffortEncoding('deepseek-v4-pro', 'openai', 'high')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
    expect(getEffortEncoding('deepseek-v4-pro', 'anthropic', 'max')).toEqual({
      thinking: { type: 'enabled' },
      output_config: { effort: 'max' },
    })
  })

  it('returns null for unknown model ids', () => {
    expect(
      getEffortEncoding('totally-unknown-model', 'openai', 'high')
    ).toBeNull()
  })
})

describe('aliases resolve reasoning config', () => {
  it('resolves grok-4-5 alias', () => {
    const entry = findModelByIdOrAlias('grok-4-5')
    expect(entry?.id).toBe('grok-4.5')
    expect(getModelReasoningConfig('grok-4-5')?.efforts).toContain('high')
  })

  it('resolves gpt-5.6-luna-pro alias', () => {
    const entry = findModelByIdOrAlias('gpt-5.6-luna-pro')
    expect(entry?.id).toBe('gpt-5.6-luna')
    expect(getSupportedEfforts('gpt-5.6-luna-pro', 'openai')).toContain('xhigh')
  })
})
