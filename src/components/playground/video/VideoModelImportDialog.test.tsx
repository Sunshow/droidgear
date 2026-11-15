import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    fetchModelsByApiKey: vi.fn(),
    getChannelApiKey: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

import type { Channel, ChannelToken } from '@/lib/bindings'
import { useChannelStore } from '@/store/channel-store'
import { render, screen, waitFor } from '@/test/test-utils'
import { VideoModelImportDialog } from './VideoModelImportDialog'

const channel: Channel = {
  id: 'channel-1',
  name: 'My Gateway',
  type: 'new-api',
  baseUrl: 'https://gateway.example',
  enabled: true,
  createdAt: 0,
}

const token: ChannelToken = {
  id: 3,
  name: 'default',
  key: 'sk-token',
  status: 1,
  remainQuota: 0,
  usedQuota: 0,
  unlimitedQuota: true,
  platform: null,
  groupName: 'vip',
}

function seedChannel() {
  useChannelStore.setState({
    channels: [channel],
    keys: { [channel.id]: [token] },
    keysFetchState: {
      [channel.id]: { isLoading: false, error: null },
    },
  })
}

describe('VideoModelImportDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useChannelStore.setState({
      channels: [],
      keys: {},
      keysFetchState: {},
    })
  })

  it('imports any model from a channel and keeps the chosen protocol', async () => {
    const user = userEvent.setup({ delay: null })
    seedChannel()
    commandMocks.fetchModelsByApiKey.mockResolvedValue({
      status: 'ok',
      data: [
        { id: 'doubao-seedance-2-0-260128', name: 'Seedance 2.0' },
        { id: 'a-custom-video-model', name: null },
      ],
    })
    const onImported = vi.fn()
    const onOpenChange = vi.fn()

    render(
      <VideoModelImportDialog
        open
        onOpenChange={onOpenChange}
        onImported={onImported}
      />
    )

    await user.click(screen.getByRole('combobox'))
    await user.click(await screen.findByRole('option', { name: /My Gateway/ }))
    await user.click(screen.getByRole('button', { name: 'Next' }))

    await user.click(await screen.findByRole('button', { name: /default/ }))

    await waitFor(() =>
      expect(screen.getByText('doubao-seedance-2-0-260128')).toBeInTheDocument()
    )
    expect(commandMocks.fetchModelsByApiKey).toHaveBeenCalledWith(
      'https://gateway.example',
      'sk-token',
      null
    )

    const [checkbox] = screen.getAllByRole('checkbox')
    expect(checkbox).toBeDefined()
    await user.click(checkbox as HTMLElement)
    await user.click(screen.getByRole('button', { name: /Import 1/ }))

    expect(onImported).toHaveBeenCalledWith([
      expect.objectContaining({
        modelId: 'doubao-seedance-2-0-260128',
        name: 'Seedance 2.0',
        protocol: 'new-api-videos',
        source: {
          kind: 'channel',
          channelId: 'channel-1',
          tokenId: 3,
        },
      }),
    ])
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })
})
