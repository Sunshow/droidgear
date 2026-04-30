import { describe, it, expect } from 'vitest'
import {
  normalizeBaseUrl,
  getProviderConfigFromPlatform,
  inferProviderFromPlatformAndModel,
  getBaseUrlForProvider,
  getBaseUrlForSub2Api,
} from './sub2api-platform'

describe('sub2api platform mapping', () => {
  it('appends /v1 for openai', () => {
    expect(normalizeBaseUrl('https://api.openai.com', '/v1')).toBe(
      'https://api.openai.com/v1'
    )
    expect(
      getProviderConfigFromPlatform('openai', 'https://api.openai.com')
    ).toEqual({
      provider: 'openai',
      baseUrl: 'https://api.openai.com/v1',
    })
  })

  it('appends /v1beta for gemini', () => {
    expect(normalizeBaseUrl('https://ai.google.dev', '/v1beta')).toBe(
      'https://ai.google.dev/v1beta'
    )
    expect(
      getProviderConfigFromPlatform('gemini', 'https://ai.google.dev')
    ).toEqual({
      provider: 'generic-chat-completion-api',
      baseUrl: 'https://ai.google.dev/v1beta',
    })
  })

  it('preserves base url for unknown platforms', () => {
    expect(
      getProviderConfigFromPlatform('unknown', 'https://example.com')
    ).toEqual({
      provider: 'generic-chat-completion-api',
      baseUrl: 'https://example.com',
    })
  })
})

describe('inferProviderFromPlatformAndModel', () => {
  it('prioritizes platform binding for known platforms', () => {
    expect(inferProviderFromPlatformAndModel('openai', 'claude-3-opus')).toBe(
      'openai'
    )
    expect(inferProviderFromPlatformAndModel('anthropic', 'gpt-4')).toBe(
      'anthropic'
    )
    expect(inferProviderFromPlatformAndModel('gemini', 'some-model')).toBe(
      'generic-chat-completion-api'
    )
  })

  it('handles antigravity platform based on model name', () => {
    expect(
      inferProviderFromPlatformAndModel('antigravity', 'claude-3-opus')
    ).toBe('anthropic')
    expect(
      inferProviderFromPlatformAndModel('antigravity', 'gemini-1.5-pro')
    ).toBe('generic-chat-completion-api')
    expect(
      inferProviderFromPlatformAndModel('antigravity', 'unknown-model')
    ).toBe('anthropic') // defaults to Claude
  })

  it('uses model name prefix matching when platform is null', () => {
    expect(inferProviderFromPlatformAndModel(null, 'claude-3-opus')).toBe(
      'anthropic'
    )
    expect(inferProviderFromPlatformAndModel(null, 'claude-sonnet-4-5')).toBe(
      'anthropic'
    )
    expect(inferProviderFromPlatformAndModel(null, 'gpt-4')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'gpt-3.5-turbo')).toBe(
      'openai'
    )
  })

  it('handles case-insensitive model name matching', () => {
    expect(inferProviderFromPlatformAndModel(null, 'GPT-5.4')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'GPT-4o')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'Claude-3-opus')).toBe(
      'anthropic'
    )
  })

  it('recognizes OpenAI o-series models', () => {
    expect(inferProviderFromPlatformAndModel(null, 'o1')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'o1-mini')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'o1-pro')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'o3')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'o3-mini')).toBe('openai')
    expect(inferProviderFromPlatformAndModel(null, 'o4-mini')).toBe('openai')
  })

  it('uses model name prefix matching when platform is unknown', () => {
    expect(inferProviderFromPlatformAndModel('unknown', 'claude-3-opus')).toBe(
      'anthropic'
    )
    expect(inferProviderFromPlatformAndModel('unknown', 'gpt-4o')).toBe(
      'openai'
    )
  })

  it('handles case-insensitive platform matching', () => {
    expect(inferProviderFromPlatformAndModel('OpenAI', 'some-model')).toBe(
      'openai'
    )
    expect(
      inferProviderFromPlatformAndModel('Anthropic', 'deepseek-v4-pro')
    ).toBe('anthropic')
    expect(
      inferProviderFromPlatformAndModel('ANTHROPIC', 'deepseek-v4-pro')
    ).toBe('anthropic')
    expect(inferProviderFromPlatformAndModel('Gemini', 'some-model')).toBe(
      'generic-chat-completion-api'
    )
    expect(
      inferProviderFromPlatformAndModel('Antigravity', 'claude-3-opus')
    ).toBe('anthropic')
  })

  it('defaults to generic for unknown platform and model', () => {
    expect(inferProviderFromPlatformAndModel(null, 'some-random-model')).toBe(
      'generic-chat-completion-api'
    )
    expect(inferProviderFromPlatformAndModel('unknown', 'custom-model')).toBe(
      'generic-chat-completion-api'
    )
  })
})

describe('getBaseUrlForProvider', () => {
  it('appends /v1 for openai provider', () => {
    expect(getBaseUrlForProvider('openai', 'https://api.example.com')).toBe(
      'https://api.example.com/v1'
    )
    expect(getBaseUrlForProvider('openai', 'https://api.example.com/')).toBe(
      'https://api.example.com/v1'
    )
  })

  it('preserves url for anthropic provider', () => {
    expect(getBaseUrlForProvider('anthropic', 'https://api.example.com')).toBe(
      'https://api.example.com'
    )
  })

  it('preserves url for generic provider', () => {
    expect(
      getBaseUrlForProvider(
        'generic-chat-completion-api',
        'https://api.example.com'
      )
    ).toBe('https://api.example.com')
  })

  it('handles antigravity platform for anthropic provider', () => {
    expect(
      getBaseUrlForProvider(
        'anthropic',
        'https://api.example.com',
        'antigravity'
      )
    ).toBe('https://api.example.com/antigravity')
  })

  it('handles antigravity platform for generic provider (Gemini)', () => {
    expect(
      getBaseUrlForProvider(
        'generic-chat-completion-api',
        'https://api.example.com',
        'antigravity'
      )
    ).toBe('https://api.example.com/antigravity/v1beta')
  })
})

describe('getBaseUrlForSub2Api', () => {
  it('does not append /v1 for openai provider', () => {
    expect(getBaseUrlForSub2Api('openai', 'https://api.example.com')).toBe(
      'https://api.example.com'
    )
    expect(getBaseUrlForSub2Api('openai', 'https://api.example.com/')).toBe(
      'https://api.example.com/'
    )
  })

  it('preserves url for anthropic provider', () => {
    expect(getBaseUrlForSub2Api('anthropic', 'https://api.example.com')).toBe(
      'https://api.example.com'
    )
  })

  it('preserves url for generic provider', () => {
    expect(
      getBaseUrlForSub2Api(
        'generic-chat-completion-api',
        'https://api.example.com'
      )
    ).toBe('https://api.example.com')
  })

  it('handles antigravity platform for anthropic provider', () => {
    expect(
      getBaseUrlForSub2Api(
        'anthropic',
        'https://api.example.com',
        'antigravity'
      )
    ).toBe('https://api.example.com/antigravity')
  })

  it('handles antigravity platform for generic provider (Gemini)', () => {
    expect(
      getBaseUrlForSub2Api(
        'generic-chat-completion-api',
        'https://api.example.com',
        'antigravity'
      )
    ).toBe('https://api.example.com/antigravity/v1beta')
  })
})
