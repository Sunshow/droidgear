import { describe, expect, it } from 'vitest'
import type { ImageProtocol } from '@/store/image-playground-store'
import {
  buildImportedImageModels,
  COMMON_IMAGE_QUALITIES,
  COMMON_IMAGE_SIZES,
  imageProtocolDefaults,
  supportsMultipleImages,
} from './image-protocol'

describe('imageProtocolDefaults', () => {
  it('suggests different values per protocol', () => {
    expect(imageProtocolDefaults('chat').sizes).toEqual(COMMON_IMAGE_SIZES.chat)
    expect(imageProtocolDefaults('images').sizes).toEqual(
      COMMON_IMAGE_SIZES.images
    )
    expect(imageProtocolDefaults('google').sizes).toEqual(
      COMMON_IMAGE_SIZES.google
    )
    expect(imageProtocolDefaults('chat').qualities).toEqual([])
    expect(imageProtocolDefaults('images').qualities).toEqual(
      COMMON_IMAGE_QUALITIES.images
    )
    expect(imageProtocolDefaults('google').qualities).toEqual([
      '1K',
      '2K',
      '4K',
    ])
  })

  it('falls back to chat suggestions for an unknown protocol', () => {
    expect(imageProtocolDefaults('mystery' as ImageProtocol).sizes).toEqual(
      COMMON_IMAGE_SIZES.chat
    )
  })
})

describe('supportsMultipleImages', () => {
  it('allows chat, Images and Imagen', () => {
    expect(
      supportsMultipleImages({ protocol: 'chat', modelId: 'any-model' })
    ).toBe(true)
    expect(
      supportsMultipleImages({
        protocol: 'images',
        modelId: 'qwen-image-2.1-turbo',
      })
    ).toBe(true)
    expect(
      supportsMultipleImages({
        protocol: 'google',
        modelId: 'imagen-4.0-generate-001',
      })
    ).toBe(true)
  })

  it('rejects Gemini image models', () => {
    expect(
      supportsMultipleImages({
        protocol: 'google',
        modelId: 'gemini-3-pro-image',
      })
    ).toBe(false)
  })
})

describe('buildImportedImageModels', () => {
  it('imports selected models with their own protocol defaults', () => {
    const protocols = new Map<string, ImageProtocol>([
      ['gemini-3-pro-image', 'google'],
      ['qwen-image-2.1-turbo', 'images'],
      ['not-selected', 'chat'],
    ])
    const drafts = buildImportedImageModels({
      models: [
        { id: 'gemini-3-pro-image', name: 'Nano Banana' },
        { id: 'qwen-image-2.1-turbo', name: null },
        { id: 'not-selected', name: null },
      ],
      selectedIds: new Set(['gemini-3-pro-image', 'qwen-image-2.1-turbo']),
      channelId: 'ch_1',
      tokenId: 7,
      protocols,
    })

    expect(drafts).toHaveLength(2)
    expect(drafts[0]).toMatchObject({
      name: 'Nano Banana',
      modelId: 'gemini-3-pro-image',
      protocol: 'google',
      source: { kind: 'channel', channelId: 'ch_1', tokenId: 7 },
      sizes: COMMON_IMAGE_SIZES.google,
      qualities: COMMON_IMAGE_QUALITIES.google,
      supportsImageInput: true,
    })
    expect(drafts[1]).toMatchObject({
      name: 'qwen-image-2.1-turbo',
      protocol: 'images',
      sizes: COMMON_IMAGE_SIZES.images,
      qualities: COMMON_IMAGE_QUALITIES.images,
    })
  })
})
