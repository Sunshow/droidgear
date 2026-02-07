import { describe, it, expect } from 'vitest'
import {
  protocolToOpenCodeNpm,
  normalizeBaseUrlForOpenCode,
} from './opencode-npm'
import type { ModelProtocol } from './types'

describe('protocolToOpenCodeNpm', () => {
  it('should map anthropic to @ai-sdk/anthropic', () => {
    const protocol: ModelProtocol = 'anthropic'
    expect(protocolToOpenCodeNpm(protocol)).toBe('@ai-sdk/anthropic')
  })

  it('should map openai to @ai-sdk/openai', () => {
    const protocol: ModelProtocol = 'openai'
    expect(protocolToOpenCodeNpm(protocol)).toBe('@ai-sdk/openai')
  })

  it('should map google-ai to @ai-sdk/google', () => {
    const protocol: ModelProtocol = 'google-ai'
    expect(protocolToOpenCodeNpm(protocol)).toBe('@ai-sdk/google')
  })

  it('should map openai-compatible to @ai-sdk/openai-compatible', () => {
    const protocol: ModelProtocol = 'openai-compatible'
    expect(protocolToOpenCodeNpm(protocol)).toBe('@ai-sdk/openai-compatible')
  })

  it('should handle all protocol types', () => {
    const protocols: ModelProtocol[] = [
      'anthropic',
      'openai',
      'google-ai',
      'openai-compatible',
    ]

    const expectedPackages = [
      '@ai-sdk/anthropic',
      '@ai-sdk/openai',
      '@ai-sdk/google',
      '@ai-sdk/openai-compatible',
    ]

    protocols.forEach((protocol, index) => {
      expect(protocolToOpenCodeNpm(protocol)).toBe(expectedPackages[index])
    })
  })
})

describe('normalizeBaseUrlForOpenCode', () => {
  it('appends /v1 for anthropic protocol', () => {
    expect(
      normalizeBaseUrlForOpenCode('anthropic', 'https://api.example.com')
    ).toBe('https://api.example.com/v1')
  })

  it('does not duplicate /v1 for anthropic', () => {
    expect(
      normalizeBaseUrlForOpenCode('anthropic', 'https://api.anthropic.com/v1')
    ).toBe('https://api.anthropic.com/v1')
  })

  it('removes trailing slashes before adding /v1', () => {
    expect(
      normalizeBaseUrlForOpenCode('anthropic', 'https://api.example.com/')
    ).toBe('https://api.example.com/v1')
  })

  it('keeps baseURL as-is for openai', () => {
    expect(
      normalizeBaseUrlForOpenCode('openai', 'https://api.openai.com')
    ).toBe('https://api.openai.com')
  })

  it('keeps baseURL as-is for google-ai', () => {
    expect(
      normalizeBaseUrlForOpenCode(
        'google-ai',
        'https://generativelanguage.googleapis.com'
      )
    ).toBe('https://generativelanguage.googleapis.com')
  })

  it('keeps baseURL as-is for openai-compatible', () => {
    expect(
      normalizeBaseUrlForOpenCode('openai-compatible', 'https://custom.api.com')
    ).toBe('https://custom.api.com')
  })
})
