import { describe, it, expect } from 'vitest'
import { inferModelProtocol, inferModelProtocolInfo } from './index'

describe('inferModelProtocol', () => {
  describe('sub-2-api channel', () => {
    it('infers from platform (highest priority)', () => {
      expect(
        inferModelProtocol('sub-2-api', 'anthropic', 'https://api.example.com')
      ).toBe('anthropic')
      expect(
        inferModelProtocol('sub-2-api', 'openai', 'https://api.example.com')
      ).toBe('openai')
      expect(
        inferModelProtocol('sub-2-api', 'gemini', 'https://api.example.com')
      ).toBe('google-ai')
      expect(
        inferModelProtocol('sub-2-api', 'grok', 'https://api.example.com')
      ).toBe('anthropic')
    })

    it('platform takes priority over model id', () => {
      // Even with a claude model, openai platform wins
      expect(
        inferModelProtocol(
          'sub-2-api',
          'openai',
          'https://api.example.com',
          'claude-3-opus'
        )
      ).toBe('openai')
      // grok platform always defaults to anthropic
      expect(
        inferModelProtocol(
          'sub-2-api',
          'grok',
          'https://api.example.com',
          'gpt-4'
        )
      ).toBe('anthropic')
    })

    it('antigravity falls through to model inference', () => {
      expect(
        inferModelProtocol(
          'sub-2-api',
          'antigravity',
          'https://api.example.com',
          'claude-3-opus'
        )
      ).toBe('anthropic')
      expect(
        inferModelProtocol(
          'sub-2-api',
          'antigravity',
          'https://api.example.com',
          'gemini-1.5-pro'
        )
      ).toBe('google-ai')
    })

    it('unknown platform falls through to global model inference', () => {
      expect(
        inferModelProtocol(
          'sub-2-api',
          null,
          'https://api.example.com',
          'claude-3-opus'
        )
      ).toBe('anthropic')
      expect(
        inferModelProtocol(
          'sub-2-api',
          null,
          'https://api.example.com',
          'gpt-4'
        )
      ).toBe('openai')
      expect(
        inferModelProtocol(
          'sub-2-api',
          null,
          'https://api.example.com',
          'gemini-pro'
        )
      ).toBe('google-ai')
    })

    it('defaults to openai-compatible without model id', () => {
      expect(
        inferModelProtocol('sub-2-api', null, 'https://api.example.com')
      ).toBe('openai-compatible')
    })
  })

  describe('new-api channel', () => {
    it('infers from model id', () => {
      expect(
        inferModelProtocol(
          'new-api',
          null,
          'https://api.example.com',
          'claude-3-opus'
        )
      ).toBe('anthropic')
      expect(
        inferModelProtocol('new-api', null, 'https://api.example.com', 'gpt-4')
      ).toBe('openai')
    })

    it('falls through to global inference for unknown models', () => {
      expect(
        inferModelProtocol(
          'new-api',
          null,
          'https://api.example.com',
          'gemini-pro'
        )
      ).toBe('google-ai')
      expect(
        inferModelProtocol(
          'new-api',
          null,
          'https://api.example.com',
          'o1-preview'
        )
      ).toBe('openai')
    })

    it('defaults to openai-compatible without model id', () => {
      expect(
        inferModelProtocol('new-api', null, 'https://api.example.com')
      ).toBe('openai-compatible')
    })
  })

  describe('cli-proxy-api channel', () => {
    it('behaves the same as new-api', () => {
      expect(
        inferModelProtocol(
          'cli-proxy-api',
          null,
          'https://api.example.com',
          'claude-3-opus'
        )
      ).toBe('anthropic')
      expect(
        inferModelProtocol(
          'cli-proxy-api',
          null,
          'https://api.example.com',
          'gpt-4'
        )
      ).toBe('openai')
      expect(
        inferModelProtocol(
          'cli-proxy-api',
          null,
          'https://api.example.com',
          'unknown-model'
        )
      ).toBe('openai-compatible')
    })
  })
})

describe('inferModelProtocolInfo', () => {
  it('returns protocol and transformed baseUrl for sub-2-api', () => {
    const info = inferModelProtocolInfo(
      'sub-2-api',
      'anthropic',
      'https://api.example.com',
      'claude-3-opus'
    )
    expect(info.protocol).toBe('anthropic')
    expect(info.baseUrl).toBe('https://api.example.com')
  })

  it('returns protocol and transformed baseUrl for new-api anthropic', () => {
    const info = inferModelProtocolInfo(
      'new-api',
      null,
      'https://api.example.com',
      'claude-3-opus'
    )
    expect(info.protocol).toBe('anthropic')
    expect(info.baseUrl).toBe('https://api.example.com')
  })

  it('returns protocol and transformed baseUrl for new-api openai', () => {
    const info = inferModelProtocolInfo(
      'new-api',
      null,
      'https://api.example.com',
      'gpt-4'
    )
    expect(info.protocol).toBe('openai')
    expect(info.baseUrl).toBe('https://api.example.com/v1')
  })

  it('handles antigravity with correct baseUrl', () => {
    const info = inferModelProtocolInfo(
      'sub-2-api',
      'antigravity',
      'https://api.example.com',
      'claude-3-opus'
    )
    expect(info.protocol).toBe('anthropic')
    expect(info.baseUrl).toBe('https://api.example.com/antigravity')
  })

  it('handles antigravity gemini with correct baseUrl', () => {
    const info = inferModelProtocolInfo(
      'sub-2-api',
      'antigravity',
      'https://api.example.com',
      'gemini-1.5-pro'
    )
    expect(info.protocol).toBe('google-ai')
    expect(info.baseUrl).toBe('https://api.example.com/antigravity/v1beta')
  })
})
