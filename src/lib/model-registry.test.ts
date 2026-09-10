import { describe, expect, it } from 'vitest'
import {
  clampEffortToSupported,
  expandOpenAiEffortEncoding,
  findModelByIdOrAlias,
  getAllRegistryModels,
  getEffortEncoding,
  getModelReasoningConfig,
  getSupportedEfforts,
  hasCustomEffortEncoding,
  isOpenAiLikeProvider,
} from './model-registry'

describe('model-registry capability coverage', () => {
  it('has capability metadata for every registered model', () => {
    const models = getAllRegistryModels()
    expect(models.length).toBeGreaterThan(0)
    for (const model of models) {
      expect(typeof model.reasoning, `${model.id} missing reasoning`).toBe(
        'boolean'
      )
      expect(model.input, `${model.id} missing input modalities`).toContain(
        'text'
      )
      expect(
        model.input.every(input => ['text', 'image'].includes(input))
      ).toBe(true)
      if (model.thinkingLevelMap) {
        expect(
          model.reasoning,
          `${model.id} maps thinking while disabled`
        ).toBe(true)
        for (const [level, value] of Object.entries(model.thinkingLevelMap)) {
          expect(
            ['off', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'],
            `${model.id} has invalid Pi thinking level ${level}`
          ).toContain(level)
          expect(
            value === null || typeof value === 'string',
            `${model.id} has invalid mapping for ${level}`
          ).toBe(true)
        }
      }
      expect(
        model.reasoningConfig,
        `${model.id} missing reasoningConfig`
      ).toBeTruthy()
      expect(model.reasoningConfig?.efforts.length).toBeGreaterThan(0)
    }
  })

  it('distinguishes reasoning and image capabilities', () => {
    expect(findModelByIdOrAlias('gpt-4o-mini')).toMatchObject({
      reasoning: false,
      input: ['text', 'image'],
    })
    expect(findModelByIdOrAlias('o3-mini')).toMatchObject({
      reasoning: true,
      input: ['text'],
    })
  })

  it('stores provider-neutral Pi thinking maps', () => {
    expect(findModelByIdOrAlias('gpt-5.6-sol')?.thinkingLevelMap).toEqual({
      minimal: null,
      xhigh: 'xhigh',
      max: 'max',
    })
    expect(findModelByIdOrAlias('deepseek-v4-pro')?.thinkingLevelMap).toEqual({
      minimal: null,
      low: null,
      medium: null,
      max: 'max',
    })
    // DeepSeek maps the requested effort per the official thinking-mode table:
    // minimal → low, medium → high, xhigh → high, max → max.
    expect(findModelByIdOrAlias('deepseek-flash')?.thinkingLevelMap).toEqual({
      minimal: 'low',
      low: 'low',
      medium: 'high',
      high: 'high',
      xhigh: 'high',
      max: 'max',
    })
    expect(
      findModelByIdOrAlias('gpt-4o-mini')?.thinkingLevelMap
    ).toBeUndefined()
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

  it('returns grok-4.6 efforts including xhigh', () => {
    expect(getSupportedEfforts('grok-4.6', 'openai')).toEqual([
      'none',
      'low',
      'medium',
      'high',
      'xhigh',
    ])
  })

  it('returns deepseek whitelist', () => {
    expect(getSupportedEfforts('deepseek-v4-pro', 'openai')).toEqual([
      'none',
      'high',
      'max',
    ])
    expect(getSupportedEfforts('deepseek-v4-flash', 'openai')).toEqual([
      'none',
      'low',
      'high',
      'max',
    ])
    expect(
      getSupportedEfforts('deepseek-v4-flash-vision-exp', 'openai')
    ).toEqual(['none', 'low', 'high', 'max'])
    expect(getSupportedEfforts('deepseek-v4.1-flash', 'openai')).toEqual([
      'none',
      'low',
      'high',
      'max',
    ])
    // the recommended model name resolves to the same entry
    expect(getSupportedEfforts('deepseek-flash', 'openai')).toEqual([
      'none',
      'low',
      'high',
      'max',
    ])
  })

  it('returns kimi-k3 efforts with max but no medium', () => {
    expect(getSupportedEfforts('kimi-k3', 'openai')).toEqual([
      'none',
      'low',
      'high',
      'max',
    ])
  })

  it('returns qwen3.8-max efforts with xhigh but no high', () => {
    expect(getSupportedEfforts('qwen3.8-max', 'openai')).toEqual([
      'none',
      'low',
      'medium',
      'xhigh',
    ])
  })

  it('returns gpt-6-astra full effort list with max', () => {
    expect(getSupportedEfforts('gpt-6-astra', 'openai')).toEqual([
      'none',
      'low',
      'medium',
      'high',
      'xhigh',
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
    expect(
      clampEffortToSupported('max', ['none', 'low', 'medium', 'xhigh'])
    ).toBe('xhigh')
  })

  it('clamps deepseek-v4-flash efforts to its whitelist', () => {
    const flashEfforts = ['none', 'low', 'high', 'max']
    expect(clampEffortToSupported('xhigh', flashEfforts)).toBe('high')
    expect(clampEffortToSupported('medium', flashEfforts)).toBe('low')
    expect(clampEffortToSupported('max', flashEfforts)).toBe('max')
    expect(clampEffortToSupported('high', flashEfforts)).toBe('high')
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

  it('encodes deepseek-v4-flash per its supported effort set', () => {
    expect(getEffortEncoding('deepseek-v4-flash', 'openai', 'none')).toEqual({
      thinking: { type: 'disabled' },
    })
    expect(getEffortEncoding('deepseek-v4-flash', 'openai', 'low')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'low',
    })
    expect(getEffortEncoding('deepseek-v4-flash', 'openai', 'high')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
    expect(getEffortEncoding('deepseek-v4-flash', 'openai', 'max')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'max',
    })
    expect(getEffortEncoding('deepseek-v4-flash', 'anthropic', 'high')).toEqual(
      {
        thinking: { type: 'enabled' },
        output_config: { effort: 'high' },
      }
    )
  })

  it('encodes deepseek-v4-flash-vision-exp like flash', () => {
    expect(
      getEffortEncoding('deepseek-v4-flash-vision-exp', 'openai', 'none')
    ).toEqual({
      thinking: { type: 'disabled' },
    })
    expect(
      getEffortEncoding('deepseek-v4-flash-vision-exp', 'openai', 'high')
    ).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
    expect(
      getEffortEncoding('deepseek-v4-flash-vision-exp', 'anthropic', 'max')
    ).toEqual({
      thinking: { type: 'enabled' },
      output_config: { effort: 'max' },
    })
  })

  it('encodes deepseek-v4.1-flash per the official thinking-mode docs', () => {
    // OpenAI format: thinking.type toggle + reasoning_effort
    expect(getEffortEncoding('deepseek-v4.1-flash', 'openai', 'none')).toEqual({
      thinking: { type: 'disabled' },
    })
    expect(getEffortEncoding('deepseek-v4.1-flash', 'openai', 'low')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'low',
    })
    expect(getEffortEncoding('deepseek-v4.1-flash', 'openai', 'high')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
    expect(getEffortEncoding('deepseek-v4.1-flash', 'openai', 'max')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'max',
    })
    // Anthropic format: thinking.type toggle + output_config.effort
    expect(
      getEffortEncoding('deepseek-v4.1-flash', 'anthropic', 'none')
    ).toEqual({
      thinking: { type: 'disabled' },
    })
    expect(
      getEffortEncoding('deepseek-v4.1-flash', 'anthropic', 'low')
    ).toEqual({
      thinking: { type: 'enabled' },
      output_config: { effort: 'low' },
    })
    expect(
      getEffortEncoding('deepseek-v4.1-flash', 'anthropic', 'max')
    ).toEqual({
      thinking: { type: 'enabled' },
      output_config: { effort: 'max' },
    })
    // deepseek-flash is the recommended model name for the same entry
    expect(getEffortEncoding('deepseek-flash', 'openai', 'high')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
    expect(hasCustomEffortEncoding('deepseek-flash', 'openai')).toBe(true)
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

  it('resolves deepseek-flash to deepseek-v4.1-flash', () => {
    expect(findModelByIdOrAlias('deepseek-flash')?.id).toBe(
      'deepseek-v4.1-flash'
    )
    expect(findModelByIdOrAlias('deepseek-v4-1-flash')?.id).toBe(
      'deepseek-v4.1-flash'
    )
    expect(findModelByIdOrAlias('deepseek-flash')).toMatchObject({
      name: 'DeepSeek V4.1 Flash',
      platform: 'openai-completions',
      reasoning: true,
      input: ['text', 'image'],
      contextWindow: 1000000,
      maxOutputTokens: 384000,
    })
  })
})

describe('openai-thinking format', () => {
  it('expands openai-thinking encoding for a level', () => {
    expect(expandOpenAiEffortEncoding('max', 'thinking')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'max',
    })
  })

  it('expands openai-thinking encoding for none as disabled', () => {
    expect(expandOpenAiEffortEncoding('none', 'thinking')).toEqual({
      thinking: { type: 'disabled' },
    })
  })

  it('expands openai-reasoning encoding for none as null', () => {
    expect(expandOpenAiEffortEncoding('none', 'reasoning')).toBeNull()
  })

  it('overrides the openai-reasoning profile with thinking format', () => {
    expect(getEffortEncoding('gpt-5.6', 'openai', 'high', 'thinking')).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
    expect(getEffortEncoding('gpt-5.6', 'openai', 'high', 'reasoning')).toEqual(
      { reasoning: { effort: 'high' } }
    )
  })

  it('keeps custom deepseek encoding regardless of format override', () => {
    expect(
      getEffortEncoding('deepseek-v4-pro', 'openai', 'high', 'thinking')
    ).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
    expect(
      getEffortEncoding('deepseek-v4-pro', 'openai', 'high', 'reasoning')
    ).toEqual({
      thinking: { type: 'enabled' },
      reasoning_effort: 'high',
    })
  })

  it('does not override anthropic profiles', () => {
    expect(
      getEffortEncoding('claude-opus-4-8', 'anthropic', 'xhigh', 'thinking')
    ).toEqual({
      thinking: { type: 'adaptive' },
      output_config: { effort: 'xhigh' },
    })
  })

  it('detects OpenAI-like providers', () => {
    expect(isOpenAiLikeProvider('openai')).toBe(true)
    expect(isOpenAiLikeProvider('generic-chat-completion-api')).toBe(true)
    expect(isOpenAiLikeProvider('anthropic')).toBe(false)
  })

  it('detects custom effort encoding', () => {
    expect(hasCustomEffortEncoding('deepseek-v4-pro', 'openai')).toBe(true)
    expect(hasCustomEffortEncoding('gpt-5.6', 'openai')).toBe(false)
    expect(hasCustomEffortEncoding('totally-unknown-model', 'openai')).toBe(
      false
    )
  })
})
