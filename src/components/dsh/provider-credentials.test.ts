import { describe, expect, it } from 'vitest'
import {
  envNameForProviderId,
  isCustomApiKeyEnv,
  isValidApiKeyEnv,
  resolveApiKeyEnv,
} from './provider-credentials'

describe('envNameForProviderId', () => {
  it('derives an upper-snake-case reference name', () => {
    expect(envNameForProviderId('my-provider')).toBe('MY_PROVIDER_API_KEY')
    expect(envNameForProviderId(' openai ')).toBe('OPENAI_API_KEY')
  })

  it('normalizes dots and other separators', () => {
    expect(envNameForProviderId('my.provider id')).toBe(
      'MY_PROVIDER_ID_API_KEY'
    )
  })

  it('returns an empty reference without a provider id', () => {
    expect(envNameForProviderId('')).toBe('')
    expect(envNameForProviderId('   ')).toBe('')
  })
})

describe('resolveApiKeyEnv', () => {
  it('uses the derived name by default', () => {
    expect(resolveApiKeyEnv('my-provider', undefined)).toBe(
      'MY_PROVIDER_API_KEY'
    )
    expect(resolveApiKeyEnv('my-provider', '')).toBe('MY_PROVIDER_API_KEY')
    expect(resolveApiKeyEnv('my-provider', '   ')).toBe('MY_PROVIDER_API_KEY')
  })

  it('prefers a trimmed custom reference', () => {
    expect(resolveApiKeyEnv('my-provider', ' CUSTOM_KEY ')).toBe('CUSTOM_KEY')
  })
})

describe('isCustomApiKeyEnv', () => {
  it('treats an empty stored reference as the derived default', () => {
    expect(isCustomApiKeyEnv('my-provider', undefined)).toBe(false)
    expect(isCustomApiKeyEnv('my-provider', '')).toBe(false)
  })

  it('detects the derived name and real overrides', () => {
    expect(isCustomApiKeyEnv('my-provider', 'MY_PROVIDER_API_KEY')).toBe(false)
    expect(isCustomApiKeyEnv('my-provider', 'CUSTOM_KEY')).toBe(true)
  })
})

describe('isValidApiKeyEnv', () => {
  it('accepts ASCII letters, digits, underscores, and hyphens', () => {
    expect(isValidApiKeyEnv('MY_PROVIDER_API_KEY')).toBe(true)
    expect(isValidApiKeyEnv('my-ref_2')).toBe(true)
  })

  it('rejects other characters', () => {
    expect(isValidApiKeyEnv('MY KEY')).toBe(false)
    expect(isValidApiKeyEnv('MY.KEY')).toBe(false)
    expect(isValidApiKeyEnv('密钥')).toBe(false)
    expect(isValidApiKeyEnv('')).toBe(false)
  })
})
