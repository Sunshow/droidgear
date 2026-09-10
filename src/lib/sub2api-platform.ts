import type { Provider } from '@/lib/bindings'

export interface ProviderConfig {
  provider: Provider
  baseUrl: string
}

/**
 * Sub2API 平台中同时支持多种 API 协议的类型。
 * 这些平台的协议由用户在添加模型时选择，而不是自动推断。
 */
const MULTI_PROTOCOL_PLATFORMS = new Set(['deepseek'])

/**
 * 多协议平台可选协议（与 Provider 一一对应，第一项为默认值）。
 * - OpenAI：Responses / Chat Completions
 * - Anthropic
 * - 通用兼容
 */
export const MULTI_PROTOCOL_PROVIDERS: Provider[] = [
  'openai',
  'anthropic',
  'generic-chat-completion-api',
]

/** 多协议平台的默认协议（OpenAI） */
export const DEFAULT_MULTI_PROTOCOL_PROVIDER: Provider = 'openai'

/** 该平台是否支持多种 API 协议（需要用户选择） */
export const isMultiProtocolPlatform = (
  platform: string | null | undefined
): boolean => !!platform && MULTI_PROTOCOL_PLATFORMS.has(platform.toLowerCase())

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
  if (platformLower === 'anthropic' || platformLower === 'grok')
    return 'anthropic'
  if (platformLower === 'gemini') return 'generic-chat-completion-api'
  // deepseek 平台同时支持三种协议，默认使用 OpenAI
  if (platformLower === 'deepseek') return 'openai'
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
  if (modelLower.startsWith('mimo-')) return 'openai'

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
  // deepseek 等平台的三种协议共用同一个 Base URL（无需 /v1、/v1beta 后缀）
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

  if (platformLower === 'anthropic' || platformLower === 'grok') {
    return { provider: 'anthropic', baseUrl }
  }

  if (platformLower === 'gemini') {
    return {
      provider: 'generic-chat-completion-api',
      baseUrl: normalizeBaseUrl(baseUrl, '/v1beta'),
    }
  }

  // deepseek 平台支持三种协议，默认 OpenAI；三种协议共用裸 Base URL
  if (platformLower === 'deepseek') {
    return { provider: 'openai', baseUrl }
  }

  return { provider: 'generic-chat-completion-api', baseUrl }
}
