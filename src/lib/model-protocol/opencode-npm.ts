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
 * because it appends /messages to make requests to /v1/messages
 *
 * Other protocols (openai, google-ai, openai-compatible) don't need this adjustment
 */
export function normalizeBaseUrlForOpenCode(
  protocol: ModelProtocol,
  baseUrl: string
): string {
  if (protocol === 'anthropic') {
    // Ensure baseURL ends with /v1 for Anthropic
    const trimmed = baseUrl.replace(/\/+$/, '') // Remove trailing slashes
    if (!trimmed.endsWith('/v1')) {
      return `${trimmed}/v1`
    }
    return trimmed
  }

  // Other protocols use baseURL as-is
  return baseUrl
}
