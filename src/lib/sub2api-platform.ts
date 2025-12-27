import type { Provider } from '@/lib/bindings'

export interface ProviderConfig {
  provider: Provider
  baseUrl: string
}

export const normalizeBaseUrl = (baseUrl: string, suffix: string): string => {
  const trimmed = baseUrl.replace(/\/+$/, '')
  if (!suffix) return trimmed
  return trimmed.endsWith(suffix) ? trimmed : `${trimmed}${suffix}`
}

export const getProviderConfigFromPlatform = (
  platform: string | null | undefined,
  baseUrl: string
): ProviderConfig => {
  if (platform === 'openai') {
    return {
      provider: 'openai',
      baseUrl: normalizeBaseUrl(baseUrl, '/v1'),
    }
  }

  if (platform === 'anthropic') {
    return { provider: 'anthropic', baseUrl }
  }

  if (platform === 'gemini') {
    return {
      provider: 'generic-chat-completion-api',
      baseUrl: normalizeBaseUrl(baseUrl, '/v1beta'),
    }
  }

  return { provider: 'generic-chat-completion-api', baseUrl }
}
