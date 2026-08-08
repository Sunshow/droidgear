import type { Provider } from '@/lib/bindings'
import { normalizeBaseUrl } from '@/lib/sub2api-platform'

export const inferProviderForNewApi = (modelId: string): Provider => {
  const modelLower = modelId.toLowerCase()
  if (modelLower.startsWith('claude-')) return 'anthropic'
  if (modelLower.startsWith('deepseek-')) return 'openai'
  if (modelLower.startsWith('mimo-')) return 'openai'
  if (modelLower.startsWith('gpt-') || /^o[134](-|$)/.test(modelLower))
    return 'openai'
  return 'generic-chat-completion-api'
}

export const getBaseUrlForNewApi = (
  provider: Provider,
  baseUrl: string
): string => {
  if (provider === 'anthropic') return baseUrl
  return normalizeBaseUrl(baseUrl, '/v1')
}
