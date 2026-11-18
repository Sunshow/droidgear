import type { TFunction } from 'i18next'
import type { VideoModelDraft, VideoProtocol } from '@/store/playground-store'

/** Suggested clip lengths. All three wires are free-input: Sora accepts
 * 4/8/12, Kling 3-30, Wan 2-30 and Seedance up to 12/15/30, so this is only a
 * shared suggestion list rather than a protocol-specific restriction. */
export const COMMON_VIDEO_DURATIONS = [2, 3, 4, 5, 6, 8, 10, 12, 15, 20, 30]

/** Suggested resolutions. Tiers (480p/720p/1080p/4K) and common WxH sizes are
 * listed together; the create form accepts free input as well. */
export const COMMON_VIDEO_RESOLUTIONS = [
  '480p',
  '720p',
  '1080p',
  '4K',
  '480x854',
  '854x480',
  '720x1280',
  '1280x720',
  '1024x1792',
  '1792x1024',
  '1080x1920',
  '1920x1080',
  '1024x1024',
  '1280x960',
  '960x1280',
  '2160x3840',
  '3840x2160',
]

export interface VideoProtocolDefaults {
  durations: number[]
  resolutions: string[]
}

/** Shared defaults for every protocol; the per-model lists can be edited. */
export function videoProtocolDefaults(
  _protocol?: VideoProtocol
): VideoProtocolDefaults {
  return {
    durations: [...COMMON_VIDEO_DURATIONS],
    resolutions: [...COMMON_VIDEO_RESOLUTIONS],
  }
}

/**
 * Options the create form's duration picker offers. The picker must never go
 * dead for a selected model: duration is free-input anyway, so a model saved
 * without its own list falls back to the shared suggestions.
 */
export function videoDurationOptions(durations: readonly number[]): string[] {
  const source = durations.length > 0 ? durations : COMMON_VIDEO_DURATIONS
  return source.map(String)
}

/** Counterpart of {@link videoDurationOptions} for resolutions. */
export function videoResolutionOptions(
  resolutions: readonly string[]
): string[] {
  return resolutions.length > 0
    ? [...resolutions]
    : [...COMMON_VIDEO_RESOLUTIONS]
}

export function videoProtocolOptions(
  t: TFunction
): { value: VideoProtocol; label: string }[] {
  return [
    { value: 'sora', label: t('playground.video.modelDialog.protocolSora') },
    {
      value: 'new-api',
      label: t('playground.video.modelDialog.protocolNewApi'),
    },
    {
      value: 'doubao',
      label: t('playground.video.modelDialog.protocolDoubao'),
    },
  ]
}

export function videoProtocolLabel(
  t: TFunction,
  protocol: VideoProtocol
): string {
  return (
    videoProtocolOptions(t).find(option => option.value === protocol)?.label ??
    t('playground.video.modelDialog.protocolSora')
  )
}

/** Builds playground models from a channel's `/v1/models` selection.
 *
 * No model id is filtered or rewritten: any model the channel lists can be
 * added. Each selected model carries its own protocol so a channel can mix
 * Sora, New API and doubao models in one import.
 */
export function buildImportedVideoModels(params: {
  models: { id: string; name: string | null }[]
  selectedIds: ReadonlySet<string>
  channelId: string
  tokenId: number | null
  protocols: ReadonlyMap<string, VideoProtocol>
}): VideoModelDraft[] {
  const defaults = videoProtocolDefaults()
  return params.models
    .filter(model => params.selectedIds.has(model.id))
    .map(model => ({
      name: model.name?.trim() || model.id,
      modelId: model.id,
      source: {
        kind: 'channel',
        channelId: params.channelId,
        tokenId: params.tokenId,
      },
      protocol: params.protocols.get(model.id) ?? 'sora',
      durations: defaults.durations,
      resolutions: defaults.resolutions,
      supportsImageInput: true,
    }))
}
