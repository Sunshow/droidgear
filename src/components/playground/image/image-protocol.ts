import type { TFunction } from 'i18next'
import type {
  ImageModelDraft,
  ImageProtocol,
} from '@/store/image-playground-store'

/** Suggested size values per protocol.
 *
 * ChatCompletions models accept either pixel sizes (`size`) or an aspect ratio
 * (`image_config.aspect_ratio`). Images models use OpenAI image sizes. Google's
 * native wire reads an aspect ratio and a separate image size. */
export const COMMON_IMAGE_SIZES: Record<ImageProtocol, string[]> = {
  chat: ['1024x1024', '1536x1024', '1024x1536', '16:9', '9:16', '4:3', '3:4'],
  images: ['auto', '1024x1024', '1536x1024', '1024x1536'],
  google: [
    '1:1',
    '2:3',
    '3:2',
    '3:4',
    '4:3',
    '4:5',
    '5:4',
    '9:16',
    '16:9',
    '21:9',
  ],
}

/** Suggested quality values per protocol. Chat models keep this in the
 * advanced JSON because every gateway names it differently. */
export const COMMON_IMAGE_QUALITIES: Record<ImageProtocol, string[]> = {
  chat: [],
  images: ['auto', 'low', 'medium', 'high'],
  google: ['1K', '2K', '4K'],
}

export interface ImageProtocolDefaults {
  sizes: string[]
  qualities: string[]
}

export function imageProtocolDefaults(
  protocol: ImageProtocol = 'chat'
): ImageProtocolDefaults {
  return {
    sizes: COMMON_IMAGE_SIZES[protocol] ?? COMMON_IMAGE_SIZES.chat,
    qualities: COMMON_IMAGE_QUALITIES[protocol] ?? [],
  }
}

export function imageProtocolOptions(
  t: TFunction
): { value: ImageProtocol; label: string }[] {
  return [
    { value: 'chat', label: t('playground.image.modelDialog.protocolChat') },
    {
      value: 'images',
      label: t('playground.image.modelDialog.protocolImages'),
    },
    {
      value: 'google',
      label: t('playground.image.modelDialog.protocolGoogle'),
    },
  ]
}

export function imageProtocolLabel(
  t: TFunction,
  protocol: ImageProtocol
): string {
  return (
    imageProtocolOptions(t).find(option => option.value === protocol)?.label ??
    t('playground.image.modelDialog.protocolChat')
  )
}

/** ChatCompletions and the Images endpoint both take `n`, and Imagen's
 * `:predict` takes `sampleCount`. Gemini's `generateContent` always answers
 * with the image the model decided to draw. */
export function supportsMultipleImages(model: {
  protocol: ImageProtocol
  modelId: string
}): boolean {
  if (model.protocol === 'chat' || model.protocol === 'images') return true
  return (
    model.protocol === 'google' &&
    model.modelId.trim().toLowerCase().startsWith('imagen')
  )
}

/** Builds playground models from a channel's `/v1/models` selection. */
export function buildImportedImageModels(params: {
  models: { id: string; name: string | null }[]
  selectedIds: ReadonlySet<string>
  channelId: string
  tokenId: number | null
  protocols: ReadonlyMap<string, ImageProtocol>
}): ImageModelDraft[] {
  return params.models
    .filter(model => params.selectedIds.has(model.id))
    .map(model => {
      const protocol = params.protocols.get(model.id) ?? 'chat'
      const defaults = imageProtocolDefaults(protocol)
      return {
        name: model.name?.trim() || model.id,
        modelId: model.id,
        source: {
          kind: 'channel',
          channelId: params.channelId,
          tokenId: params.tokenId,
        },
        protocol,
        sizes: defaults.sizes,
        qualities: defaults.qualities,
        supportsImageInput: true,
      }
    })
}
