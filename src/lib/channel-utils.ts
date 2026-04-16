import { commands, type ChannelType } from '@/lib/bindings'

export function isApiKeyAuthChannel(type: ChannelType): boolean {
  return type === 'cli-proxy-api' || type === 'ollama' || type === 'general'
}

/** Platforms known to be incompatible with OpenAI API format */
const NON_OPENAI_PLATFORMS = new Set(['gemini', 'anthropic', 'claude'])

/**
 * Returns true if the platform uses OpenAI-compatible API format.
 * null/undefined is treated as OpenAI-compatible (default).
 */
export function isOpenAICompatiblePlatform(
  platform: string | null | undefined
): boolean {
  if (!platform) return true
  return !NON_OPENAI_PLATFORMS.has(platform)
}

export async function saveChannelAuth(
  channelId: string,
  channelType: ChannelType,
  username: string,
  password: string
): Promise<{ ok: boolean; error?: string }> {
  if (isApiKeyAuthChannel(channelType)) {
    const result = await commands.saveChannelApiKey(channelId, password)
    if (result.status !== 'ok') return { ok: false, error: result.error }
  } else {
    const result = await commands.saveChannelCredentials(
      channelId,
      username,
      password
    )
    if (result.status !== 'ok') return { ok: false, error: result.error }
  }
  return { ok: true }
}
