import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@/test/test-utils'
import type { Channel } from '@/lib/bindings'
import { useChannelStore } from '@/store/channel-store'
import { VideoModelDialog } from './VideoModelDialog'
import {
  COMMON_VIDEO_DURATIONS,
  COMMON_VIDEO_RESOLUTIONS,
} from './video-protocol'

const channel: Channel = {
  id: 'channel-1',
  name: 'My Gateway',
  type: 'new-api',
  baseUrl: 'https://gateway.example',
  enabled: true,
  createdAt: 0,
}

describe('VideoModelDialog', () => {
  beforeEach(() => {
    useChannelStore.setState({ channels: [channel], isLoading: false })
  })

  it('adds a Sora model with the shared option suggestions', async () => {
    const onSave = vi.fn()
    const onOpenChange = vi.fn()
    const user = userEvent.setup({ delay: null })
    render(
      <VideoModelDialog
        open
        onOpenChange={onOpenChange}
        onSave={onSave}
        onSaveMany={vi.fn()}
      />
    )

    await user.type(screen.getByLabelText('Display name'), 'Seedance')
    await user.type(
      screen.getByLabelText('Model ID'),
      'doubao-seedance-2-0-mini-260615'
    )
    await user.click(screen.getByRole('combobox', { name: 'Channel' }))
    await user.click(await screen.findByRole('option', { name: /My Gateway/ }))
    await user.click(screen.getByRole('button', { name: 'Add' }))

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        name: 'Seedance',
        protocol: 'sora',
        source: {
          kind: 'channel',
          channelId: 'channel-1',
          tokenId: null,
        },
        durations: COMMON_VIDEO_DURATIONS,
        resolutions: COMMON_VIDEO_RESOLUTIONS,
      })
    )
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it('saves edited duration and resolution lists', async () => {
    const onSave = vi.fn()
    const user = userEvent.setup({ delay: null })
    render(
      <VideoModelDialog
        open
        onOpenChange={vi.fn()}
        model={{
          id: 'vm_1',
          name: 'Seedance',
          modelId: 'doubao-seedance-2-0-mini-260615',
          source: { kind: 'channel', channelId: 'channel-1', tokenId: 1 },
          protocol: 'doubao',
          durations: [5, 10],
          resolutions: ['720p'],
          supportsImageInput: true,
        }}
        onSave={onSave}
        onSaveMany={vi.fn()}
      />
    )

    const durations = screen.getByLabelText('Durations (seconds)')
    await user.clear(durations)
    await user.type(durations, '4, 8, 12')
    const resolutions = screen.getByLabelText('Resolutions')
    await user.clear(resolutions)
    await user.type(resolutions, '1080p, 1920x1080')
    await user.click(screen.getByRole('button', { name: 'Save' }))

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        durations: [4, 8, 12],
        resolutions: ['1080p', '1920x1080'],
        protocol: 'doubao',
      })
    )
  })
})
