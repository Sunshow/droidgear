import { describe, it, expect } from 'vitest'
import {
  normalizeBaseUrl,
  getProviderConfigFromPlatform,
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
