import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Channel } from '@/lib/bindings'
import { useChannelStore } from '@/store/channel-store'
import { usePlaygroundStore, type VideoModel } from '@/store/playground-store'
import { render, screen } from '@/test/test-utils'
import { VideoModelManagerDialog } from './VideoModelManagerDialog'

const channel: Channel = {
  id: 'channel-1',
  name: 'My Gateway',
  type: 'new-api',
  baseUrl: 'https://gateway.example',
  enabled: true,
  createdAt: 0,
}

const model: VideoModel = {
  id: 'vm_1',
  name: 'Seedance',
  modelId: 'doubao-seedance-2-0-mini-260615',
  source: { kind: 'channel', channelId: channel.id, tokenId: 1 },
  protocol: 'sora',
  durations: [5],
  resolutions: ['720p'],
  supportsImageInput: true,
}

describe('VideoModelManagerDialog', () => {
  beforeEach(() => {
    useChannelStore.setState({ channels: [channel], isLoading: false })
    usePlaygroundStore.setState({ videoModels: [model] })
  })

  it('shows the channel name instead of a removed-channel label', () => {
    render(<VideoModelManagerDialog open onOpenChange={vi.fn()} />)

    expect(screen.getByText(/My Gateway/)).toBeInTheDocument()
    expect(screen.queryByText(/Channel removed/)).not.toBeInTheDocument()
  })

  it('loads channels before labeling channel-backed models', async () => {
    const loadChannels = vi.fn(async () => {
      useChannelStore.setState({ channels: [channel], isLoading: false })
    })
    useChannelStore.setState({
      channels: [],
      isLoading: false,
      loadChannels,
    })

    render(<VideoModelManagerDialog open onOpenChange={vi.fn()} />)

    expect(await screen.findByText(/My Gateway/)).toBeInTheDocument()
    expect(screen.queryByText(/Channel removed/)).not.toBeInTheDocument()
    expect(loadChannels).toHaveBeenCalled()
  })
})
