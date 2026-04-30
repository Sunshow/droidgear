import type { Provider } from '@/lib/bindings'

export interface ProviderConfig {
  provider: Provider
  baseUrl: string
}

/**
 * Infer the provider type based on platform and model ID.
 * Priority: platform binding > model name prefix matching > generic
 */
export const inferProviderFromPlatformAndModel = (
  platform: string | null | undefined,
  modelId: string
): Provider => {
  const platformLower = platform?.toLowerCase()

  // 1. Platform-based binding (highest priority)
  if (platformLower === 'openai') return 'openai'
  if (platformLower === 'anthropic') return 'anthropic'
  if (platformLower === 'gemini') return 'generic-chat-completion-api'
  if (platformLower === 'antigravity') {
    // Antigravity supports both Claude and Gemini, infer from model name
    const lower = modelId.toLowerCase()
    if (lower.startsWith('claude-')) return 'anthropic'
    if (lower.startsWith('gemini-')) return 'generic-chat-completion-api'
    return 'anthropic' // Default to Claude
  }

  // 2. Model name prefix matching (case-insensitive)
  const modelLower = modelId.toLowerCase()
  if (modelLower.startsWith('claude-')) return 'anthropic'
  if (modelLower.startsWith('gpt-') || /^o[134](-|$)/.test(modelLower))
    return 'openai'

  // 3. Default to generic
  return 'generic-chat-completion-api'
}

/**
 * Get the base URL for a provider, applying necessary normalizations
 */
export const getBaseUrlForProvider = (
  provider: Provider,
  baseUrl: string,
  platform?: string | null
): string => {
  if (platform === 'antigravity') {
    if (provider === 'anthropic') {
      return normalizeBaseUrl(baseUrl, '/antigravity')
    }
    if (provider === 'generic-chat-completion-api') {
      return normalizeBaseUrl(baseUrl, '/antigravity/v1beta')
    }
  }
  if (provider === 'openai') {
    return normalizeBaseUrl(baseUrl, '/v1')
  }
  return baseUrl
}

export const getBaseUrlForSub2Api = (
  provider: Provider,
  baseUrl: string,
  platform?: string | null
): string => {
  if (platform === 'antigravity') {
    if (provider === 'anthropic') {
      return normalizeBaseUrl(baseUrl, '/antigravity')
    }
    if (provider === 'generic-chat-completion-api') {
      return normalizeBaseUrl(baseUrl, '/antigravity/v1beta')
    }
  }
  return baseUrl
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
  const platformLower = platform?.toLowerCase()

  if (platformLower === 'openai') {
    return {
      provider: 'openai',
      baseUrl: normalizeBaseUrl(baseUrl, '/v1'),
    }
  }

  if (platformLower === 'anthropic') {
    return { provider: 'anthropic', baseUrl }
  }

  if (platformLower === 'gemini') {
    return {
      provider: 'generic-chat-completion-api',
      baseUrl: normalizeBaseUrl(baseUrl, '/v1beta'),
    }
  }

  return { provider: 'generic-chat-completion-api', baseUrl }
}
