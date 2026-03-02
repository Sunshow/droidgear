import { commands, type ChannelType } from '@/lib/bindings'

export function isApiKeyAuthChannel(type: ChannelType): boolean {
  return type === 'cli-proxy-api' || type === 'ollama' || type === 'general'
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
