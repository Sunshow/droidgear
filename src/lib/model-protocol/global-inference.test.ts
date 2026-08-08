import { describe, it, expect } from 'vitest'
import { inferProtocolFromModelId } from './global-inference'

describe('inferProtocolFromModelId', () => {
  it('identifies Claude models as anthropic', () => {
    expect(inferProtocolFromModelId('claude-3-opus')).toBe('anthropic')
    expect(inferProtocolFromModelId('claude-sonnet-4')).toBe('anthropic')
    expect(inferProtocolFromModelId('claude-3-haiku')).toBe('anthropic')
  })

  it('identifies GPT models as openai', () => {
    expect(inferProtocolFromModelId('gpt-4')).toBe('openai')
    expect(inferProtocolFromModelId('gpt-5-turbo')).toBe('openai')
    expect(inferProtocolFromModelId('gpt-3.5-turbo')).toBe('openai')
  })

  it('identifies o1/o3 models as openai', () => {
    expect(inferProtocolFromModelId('o1-preview')).toBe('openai')
    expect(inferProtocolFromModelId('o1-mini')).toBe('openai')
    expect(inferProtocolFromModelId('o3-mini')).toBe('openai')
  })

  it('identifies Gemini models as google-ai', () => {
    expect(inferProtocolFromModelId('gemini-pro')).toBe('google-ai')
    expect(inferProtocolFromModelId('gemini-1.5-pro')).toBe('google-ai')
    expect(inferProtocolFromModelId('gemini-2-flash')).toBe('google-ai')
  })

  it('identifies DeepSeek models as openai', () => {
    expect(inferProtocolFromModelId('deepseek-v3')).toBe('openai')
    expect(inferProtocolFromModelId('deepseek-v4-pro')).toBe('openai')
    expect(inferProtocolFromModelId('deepseek-chat')).toBe('openai')
  })

  it('identifies MiMo models as openai', () => {
    expect(inferProtocolFromModelId('mimo-v2.5')).toBe('openai')
    expect(inferProtocolFromModelId('mimo-v2.5-pro')).toBe('openai')
    expect(inferProtocolFromModelId('mimo-v2-flash')).toBe('openai')
  })

  it('returns openai-compatible for unknown models', () => {
    expect(inferProtocolFromModelId('llama-3')).toBe('openai-compatible')
    expect(inferProtocolFromModelId('unknown-model')).toBe('openai-compatible')
  })

  it('handles case-insensitive matching', () => {
    expect(inferProtocolFromModelId('Claude-3-opus')).toBe('anthropic')
    expect(inferProtocolFromModelId('GPT-4')).toBe('openai')
    expect(inferProtocolFromModelId('Gemini-Pro')).toBe('google-ai')
  })
})
