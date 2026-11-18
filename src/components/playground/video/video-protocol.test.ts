import { describe, expect, it } from 'vitest'
import type { VideoProtocol } from '@/store/playground-store'
import {
  buildImportedVideoModels,
  COMMON_VIDEO_DURATIONS,
  COMMON_VIDEO_RESOLUTIONS,
  videoDurationOptions,
  videoProtocolDefaults,
  videoResolutionOptions,
} from './video-protocol'

function protocolMap(
  ids: string[],
  protocol: VideoProtocol
): Map<string, VideoProtocol> {
  return new Map(ids.map(id => [id, protocol]))
}

describe('videoProtocolDefaults', () => {
  it('uses the same suggestion lists for every protocol', () => {
    const sora = videoProtocolDefaults('sora')
    const newApi = videoProtocolDefaults('new-api')
    const doubao = videoProtocolDefaults('doubao')

    expect(sora.durations).toEqual(COMMON_VIDEO_DURATIONS)
    expect(newApi.durations).toEqual(COMMON_VIDEO_DURATIONS)
    expect(doubao.resolutions).toEqual(COMMON_VIDEO_RESOLUTIONS)
    expect(sora.resolutions).toEqual(COMMON_VIDEO_RESOLUTIONS)
  })

  it('lists both resolution tiers and WxH sizes', () => {
    expect(COMMON_VIDEO_RESOLUTIONS).toContain('720p')
    expect(COMMON_VIDEO_RESOLUTIONS).toContain('1280x720')
    expect(COMMON_VIDEO_RESOLUTIONS).toContain('1024x1792')
  })
})

describe('picker suggestions', () => {
  it('keeps the model lists when the model defines them', () => {
    expect(videoDurationOptions([4, 8, 12])).toEqual(['4', '8', '12'])
    expect(videoResolutionOptions(['1080p', '1920x1080'])).toEqual([
      '1080p',
      '1920x1080',
    ])
  })

  it('falls back to the shared lists when the model has none', () => {
    expect(videoDurationOptions([])).toEqual(COMMON_VIDEO_DURATIONS.map(String))
    expect(videoResolutionOptions([])).toEqual(COMMON_VIDEO_RESOLUTIONS)
  })
})

describe('buildImportedVideoModels', () => {
  it('imports every selected model without filtering its id', () => {
    const ids = ['doubao-seedance-2-0-mini-260615', 'some-custom-video-model']
    const drafts = buildImportedVideoModels({
      models: [
        { id: 'doubao-seedance-2-0-mini-260615', name: 'Seedance Mini' },
        { id: 'some-custom-video-model', name: null },
        { id: 'not-selected', name: null },
      ],
      selectedIds: new Set(ids),
      channelId: 'ch_1',
      tokenId: 7,
      protocols: protocolMap(ids, 'doubao'),
    })

    expect(drafts).toHaveLength(2)
    expect(drafts[0]).toMatchObject({
      name: 'Seedance Mini',
      modelId: 'doubao-seedance-2-0-mini-260615',
      protocol: 'doubao',
      source: { kind: 'channel', channelId: 'ch_1', tokenId: 7 },
      supportsImageInput: true,
    })
    expect(drafts[1]).toMatchObject({
      name: 'some-custom-video-model',
      modelId: 'some-custom-video-model',
    })
  })

  it('keeps the selected protocol and shared options for the batch', () => {
    const drafts = buildImportedVideoModels({
      models: [{ id: 'video-1', name: null }],
      selectedIds: new Set(['video-1']),
      channelId: 'ch_1',
      tokenId: null,
      protocols: protocolMap(['video-1'], 'new-api'),
    })

    expect(drafts[0]?.protocol).toBe('new-api')
    expect(drafts[0]?.durations).toEqual(COMMON_VIDEO_DURATIONS)
    expect(drafts[0]?.resolutions).toEqual(COMMON_VIDEO_RESOLUTIONS)
    expect(drafts[0]?.supportsImageInput).toBe(true)
  })

  it('allows a different protocol per model in the same import', () => {
    const protocols = new Map<string, VideoProtocol>([
      ['doubao-seedance-2-0-260128', 'doubao'],
      ['sora-2', 'sora'],
      ['kling-v3', 'new-api'],
    ])
    const drafts = buildImportedVideoModels({
      models: [
        { id: 'doubao-seedance-2-0-260128', name: null },
        { id: 'sora-2', name: null },
        { id: 'kling-v3', name: null },
      ],
      selectedIds: new Set(protocols.keys()),
      channelId: 'ch_1',
      tokenId: null,
      protocols,
    })

    expect(drafts.map(draft => draft.protocol)).toEqual([
      'doubao',
      'sora',
      'new-api',
    ])
  })
})
