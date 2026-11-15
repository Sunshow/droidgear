import type { VideoModelDraft, VideoProtocol } from '@/store/playground-store'

export interface VideoProtocolDefaults {
  durations: number[]
  resolutions: string[]
}

/** Sensible create-form options for a protocol.
 *
 * Doubao expects resolution tiers (480p/720p/1080p); the OpenAI Videos wire
 * carries a size such as 1280x720. Both are only defaults: every model stores
 * the options the user configured.
 */
export function videoProtocolDefaults(
  protocol: VideoProtocol
): VideoProtocolDefaults {
  switch (protocol) {
    case 'doubao':
      return {
        durations: [5, 10],
        resolutions: ['480p', '720p', '1080p'],
      }
    case 'new-api-videos':
    default:
      return {
        durations: [5, 10],
        resolutions: ['1280x720', '720x1280'],
      }
  }
}

/** Builds playground models from a channel's `/v1/models` selection.
 *
 * No model id is filtered or rewritten: any model the channel lists can be
 * added. Each selected model carries its own protocol so a channel can mix
 * doubao and generic video models in one import.
 */
export function buildImportedVideoModels(params: {
  models: { id: string; name: string | null }[]
  selectedIds: ReadonlySet<string>
  channelId: string
  tokenId: number | null
  protocols: ReadonlyMap<string, VideoProtocol>
}): VideoModelDraft[] {
  return params.models
    .filter(model => params.selectedIds.has(model.id))
    .map(model => {
      const protocol = params.protocols.get(model.id) ?? 'new-api-videos'
      const defaults = videoProtocolDefaults(protocol)
      return {
        name: model.name?.trim() || model.id,
        modelId: model.id,
        source: {
          kind: 'channel',
          channelId: params.channelId,
          tokenId: params.tokenId,
        },
        protocol,
        durations: defaults.durations,
        resolutions: defaults.resolutions,
        supportsImageInput: protocol === 'doubao',
      }
    })
}
