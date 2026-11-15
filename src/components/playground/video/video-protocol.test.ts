import { describe, expect, it } from 'vitest'
import type { VideoProtocol } from '@/store/playground-store'
import {
  buildImportedVideoModels,
  videoProtocolDefaults,
} from './video-protocol'

function protocolMap(
  ids: string[],
  protocol: VideoProtocol
): Map<string, VideoProtocol> {
  return new Map(ids.map(id => [id, protocol]))
}

describe('videoProtocolDefaults', () => {
  it('uses Ark resolution tiers for doubao', () => {
    const defaults = videoProtocolDefaults('doubao')
    expect(defaults.resolutions).toContain('720p')
    expect(defaults.resolutions).toContain('1080p')
  })

  it('uses sizes for the New API videos wire', () => {
    const defaults = videoProtocolDefaults('new-api-videos')
    expect(defaults.resolutions).toContain('1280x720')
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

  it('keeps the selected protocol for the whole batch', () => {
    const drafts = buildImportedVideoModels({
      models: [{ id: 'video-1', name: null }],
      selectedIds: new Set(['video-1']),
      channelId: 'ch_1',
      tokenId: null,
      protocols: protocolMap(['video-1'], 'new-api-videos'),
    })

    expect(drafts[0]?.protocol).toBe('new-api-videos')
    expect(drafts[0]?.resolutions).toEqual(
      videoProtocolDefaults('new-api-videos').resolutions
    )
    expect(drafts[0]?.supportsImageInput).toBe(false)
  })

  it('allows a different protocol per model in the same import', () => {
    const protocols = new Map<string, VideoProtocol>([
      ['doubao-seedance-2-0-260128', 'doubao'],
      ['sora-2', 'new-api-videos'],
    ])
    const drafts = buildImportedVideoModels({
      models: [
        { id: 'doubao-seedance-2-0-260128', name: null },
        { id: 'sora-2', name: null },
      ],
      selectedIds: new Set(protocols.keys()),
      channelId: 'ch_1',
      tokenId: null,
      protocols,
    })

    expect(drafts.map(draft => draft.protocol)).toEqual([
      'doubao',
      'new-api-videos',
    ])
    expect(drafts[0]?.resolutions).toContain('720p')
    expect(drafts[1]?.resolutions).toContain('1280x720')
  })
})
