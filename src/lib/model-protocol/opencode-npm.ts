import type { ModelProtocol } from './types'

/**
 * Map ModelProtocol to OpenCode npm package name
 *
 * OpenCode uses AI SDK packages for different providers:
 * - Anthropic: @ai-sdk/anthropic
 * - OpenAI: @ai-sdk/openai
 * - Google AI: @ai-sdk/google
 * - Generic OpenAI-compatible: @ai-sdk/openai-compatible
 */
export function protocolToOpenCodeNpm(protocol: ModelProtocol): string {
  switch (protocol) {
    case 'anthropic':
      return '@ai-sdk/anthropic'
    case 'openai':
      return '@ai-sdk/openai'
    case 'google-ai':
      return '@ai-sdk/google'
    case 'openai-compatible':
      return '@ai-sdk/openai-compatible'
  }
}

/**
 * Normalize baseURL for OpenCode provider based on protocol
 *
 * @ai-sdk/anthropic expects baseURL to include /v1 (e.g., https://api.anthropic.com/v1)
 * @ai-sdk/openai expects baseURL to include /v1 (e.g., https://api.openai.com/v1)
 * @ai-sdk/google expects baseURL to include /v1beta (e.g., https://generativelanguage.googleapis.com/v1beta)
 */
export function normalizeBaseUrlForOpenCode(
  protocol: ModelProtocol,
  baseUrl: string
): string {
  const trimmed = baseUrl.replace(/\/+$/, '')

  if (protocol === 'anthropic' || protocol === 'openai') {
    if (!trimmed.endsWith('/v1')) {
      return `${trimmed}/v1`
    }
    return trimmed
  }

  if (protocol === 'google-ai') {
    if (!trimmed.endsWith('/v1beta')) {
      return `${trimmed}/v1beta`
    }
    return trimmed
  }

  // openai-compatible uses baseURL as-is
  return baseUrl
}
